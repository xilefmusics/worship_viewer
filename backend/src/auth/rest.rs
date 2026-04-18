use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite},
    post,
    web::{self, Data},
};
use time::Duration as CookieDuration;
use tracing::warn;

pub use super::authorization_bearer;
use super::{oidc, otp};
use crate::resources::user::session::service::SessionServiceHandle;
use crate::settings::CookieConfig;

pub fn scope(auth_rate_limit_rps: u64, auth_rate_limit_burst: u32) -> actix_web::Scope {
    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_second(auth_rate_limit_rps)
        .burst_size(auth_rate_limit_burst)
        .finish()
        .expect("valid rate-limit configuration");

    actix_web::web::scope("/auth")
        // OIDC callback is not rate-limited — it is initiated by the provider and must not block.
        .service(oidc::rest::callback)
        .service(
            web::scope("")
                .wrap(Governor::new(&governor_conf))
                .service(oidc::rest::login)
                .service(otp::rest::otp_request)
                .service(otp::rest::otp_verify)
                .service(logout),
        )
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses((status = 204, description = "Ends the session idempotently: clears `sso_session` cookie and deletes the session server-side if the cookie or `Authorization: Bearer` session id is present. Missing/unknown sessions still yield 204.")),
    tag = "Auth",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("/logout")]
async fn logout(
    svc: Data<SessionServiceHandle>,
    cookie_cfg: Data<CookieConfig>,
    req: HttpRequest,
) -> HttpResponse {
    let bearer_session = authorization_bearer(&req);
    let cookie_session = req
        .cookie(&cookie_cfg.name)
        .map(|cookie| cookie.value().to_owned());

    if let Some(session_id) = bearer_session.as_deref().or(cookie_session.as_deref())
        && let Err(err) = svc.delete_session(session_id).await
    {
        warn!(session = session_id, "failed to drop session: {}", err);
    }

    let mut response = HttpResponse::NoContent();
    if cookie_session.is_some() {
        response.cookie(empty_cookie(&cookie_cfg));
    }
    response.finish()
}

fn empty_cookie(cfg: &CookieConfig) -> Cookie<'static> {
    Cookie::build(cfg.name.clone(), "")
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(cfg.secure)
        .max_age(CookieDuration::seconds(0))
        .finish()
}
