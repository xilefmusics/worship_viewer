use std::sync::Arc;

use actix_web::{
    HttpResponse,
    cookie::{Cookie, SameSite},
    get,
    http::header,
    web::{self, Data},
};
use chrono::{Duration as ChronoDuration, Utc};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge};
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::reqwest::async_http_client;
use openidconnect::{Nonce, Scope};
use serde::Deserialize;
use time::Duration as CookieDuration;
use utoipa::IntoParams;

use super::{Model as OidcModel, OidcClients, OidcProvider, PendingOidc};
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::resources::Session;
use crate::resources::user::service::UserServiceHandle;
use crate::resources::user::session::service::SessionServiceHandle;
use crate::settings::CookieConfig;

#[utoipa::path(
    get,
    path = "/auth/login",
    params(LoginQuery),
    responses(
        (status = 302, description = "Redirect to OIDC provider login page"),
        (status = 400, description = "Invalid login request", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Rate limit exceeded; slow down and retry", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to prepare login flow", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Auth"
)]
#[get("/login")]
async fn login(
    db: Data<Database>,
    oidc_clients: Data<Arc<OidcClients>>,
    query: web::Query<LoginQuery>,
) -> Result<HttpResponse, AppError> {
    db.cleanup_expired_oidc_states().await?;
    let redirect_hint = query.redirect_to.as_deref().and_then(sanitize_redirect);

    let oidc_clients = oidc_clients.get_ref();
    let provider = OidcProvider::Google;
    let registration = oidc_clients
        .get(&provider)
        .ok_or_else(|| AppError::invalid_request("oauth provider not configured"))?;
    let oidc_client = registration.client();
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_url = oidc_client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );
    auth_url = auth_url.set_pkce_challenge(challenge);
    for scope in registration.scopes() {
        auth_url = auth_url.add_scope(Scope::new(scope.into()));
    }

    let (url, csrf, nonce) = auth_url.url();
    let now = Utc::now();
    let expires_at = now + ChronoDuration::seconds(600);
    db.remember_oidc_state(
        csrf.secret(),
        PendingOidc {
            pkce_verifier: verifier,
            nonce,
            redirect_to: redirect_hint,
            created_at: now,
            expires_at,
            provider,
        },
    )
    .await?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, url.as_ref()))
        .finish())
}

#[utoipa::path(
    get,
    path = "/auth/callback",
    params(AuthCallbackQuery),
    responses(
        (status = 302, description = "Successful callback exchange; redirects back to frontend"),
        (status = 400, description = "Invalid OIDC state", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "OIDC user info missing required claims", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "OIDC provider or database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Auth"
)]
#[get("/callback")]
async fn callback(
    db: Data<Database>,
    user_svc: Data<UserServiceHandle>,
    session_svc: Data<SessionServiceHandle>,
    oidc_clients: Data<Arc<OidcClients>>,
    cookie_cfg: Data<CookieConfig>,
    query: web::Query<AuthCallbackQuery>,
) -> Result<HttpResponse, AppError> {
    db.cleanup_expired_oidc_states().await?;
    let pending = db
        .take_oidc_state(&query.state)
        .await?
        .ok_or_else(AppError::invalid_state)?;

    let PendingOidc {
        pkce_verifier,
        nonce,
        redirect_to,
        created_at: _,
        expires_at: _,
        provider,
    } = pending;

    let oidc_clients = oidc_clients.get_ref();
    let registration = oidc_clients
        .get(&provider)
        .ok_or_else(|| AppError::invalid_request("oauth provider not configured"))?;
    let oidc_client = registration.client();

    let mut token_request = oidc_client.exchange_code(AuthorizationCode::new(query.code.clone()));
    token_request = token_request.set_pkce_verifier(pkce_verifier);

    let token_response = token_request
        .request_async(async_http_client)
        .await
        .map_err(|e| crate::log_and_convert!(AppError::oidc, "oidc.token_exchange", e))?;

    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or_else(|| AppError::invalid_request("provider response missing id_token"))?;

    let claims = id_token
        .claims(&oidc_client.id_token_verifier(), &nonce)
        .map_err(|e| crate::log_and_convert!(AppError::oidc, "oidc.id_token_claims", e))?;

    let user = user_svc
        .get_user_by_email_or_create(claims.email().ok_or(AppError::Unauthorized)?)
        .await?;
    let session = session_svc
        .create_session(Session::new(user, cookie_cfg.session_ttl_seconds as i64))
        .await?;
    let redirect_target =
        resolve_frontend_redirect(&cookie_cfg.post_login_path, redirect_to.as_deref());

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, redirect_target))
        .cookie(session_cookie(&session.id, &cookie_cfg))
        .finish())
}

fn session_cookie(session_id: &str, cfg: &CookieConfig) -> Cookie<'static> {
    let mut builder = Cookie::build(cfg.name.clone(), session_id.to_owned())
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(cfg.secure);

    if cfg.session_ttl_seconds > 0 {
        builder = builder.max_age(CookieDuration::seconds(cfg.session_ttl_seconds as i64));
    }

    builder.finish()
}

fn sanitize_redirect(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || !trimmed.starts_with('/')
        || trimmed.starts_with("//")
        || trimmed.starts_with("/http")
    {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct LoginQuery {
    /// Optional same-origin path (`/…`) to use after login. Must start with `/` and must not be `//…` or `/http…`; otherwise it is ignored and the default post-login path is used (see `sanitize_redirect`).
    #[serde(default)]
    #[param(required = false)]
    redirect_to: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AuthCallbackQuery {
    #[param(required = true)]
    code: String,
    #[param(required = true)]
    state: String,
}

fn resolve_frontend_redirect(post_login_path: &str, requested: Option<&str>) -> String {
    requested
        .and_then(sanitize_redirect)
        .unwrap_or_else(|| default_frontend_path(post_login_path))
}

fn default_frontend_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else {
        format!("/{}", trimmed.trim_start_matches('/'))
    }
}
