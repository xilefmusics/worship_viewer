use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite},
    post,
    web::Data,
};
use time::Duration as CookieDuration;
use tracing::warn;

pub use super::authorization_bearer;
use super::{oidc, otp};
use crate::resources::user::session::service::SessionServiceHandle;
use crate::settings::CookieConfig;

pub fn scope() -> actix_web::Scope {
    actix_web::web::scope("/auth")
        .service(oidc::rest::login)
        .service(oidc::rest::callback)
        .service(otp::rest::otp_request)
        .service(otp::rest::otp_verify)
        .service(logout)
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses((status = 204, description = "Session cookie cleared")),
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

pub(crate) fn empty_cookie(cfg: &CookieConfig) -> Cookie<'static> {
    Cookie::build(cfg.name.clone(), "")
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(cfg.secure)
        .max_age(CookieDuration::seconds(0))
        .finish()
}
