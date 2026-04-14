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
use crate::settings::Settings;

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
async fn logout(svc: Data<SessionServiceHandle>, req: HttpRequest) -> HttpResponse {
    let settings = Settings::global();
    let bearer_session = authorization_bearer(&req);
    let cookie_session = req
        .cookie(&settings.cookie_name)
        .map(|cookie| cookie.value().to_owned());

    if let Some(session_id) = bearer_session.as_deref().or(cookie_session.as_deref())
        && let Err(err) = svc.delete_session(session_id).await
    {
        warn!(session = session_id, "failed to drop session: {}", err);
    }

    let mut response = HttpResponse::NoContent();
    if cookie_session.is_some() {
        response.cookie(empty_cookie());
    }
    response.finish()
}

fn empty_cookie() -> Cookie<'static> {
    let settings = Settings::global();
    Cookie::build(settings.cookie_name.clone(), "")
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(settings.cookie_secure)
        .max_age(CookieDuration::seconds(0))
        .finish()
}
