use actix_web::{
    HttpResponse,
    cookie::{Cookie, SameSite},
    post,
    web::{self, Data},
};

use rand::Rng;
use shared::auth::otp::{OtpRequest, OtpVerify};
use time::Duration as CookieDuration;

use super::Model;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::mail::MailService;
use crate::resources::user::service::UserServiceHandle;
use crate::resources::user::session::service::SessionServiceHandle;
use crate::settings::{CookieConfig, OtpConfig};
use shared::user::{Session, SessionBody};

#[utoipa::path(
    post,
    path = "/auth/otp/request",
    request_body = OtpRequest,
    responses(
        (status = 204, description = "OTP generated and delivered out-of-band. Rate limits apply per IP (see server `auth_rate_limit_*` settings). Lockout after too many failed verify attempts is enforced on `/auth/otp/verify`."),
        (status = 400, description = "Email missing or invalid", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Rate limit exceeded; slow down and retry", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to persist or deliver OTP", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Auth"
)]
#[post("/otp/request")]
async fn otp_request(
    db: Data<Database>,
    mail: Data<MailService>,
    otp_cfg: Data<OtpConfig>,
    payload: web::Json<OtpRequest>,
) -> Result<HttpResponse, AppError> {
    let email = payload
        .email
        .trim()
        .to_lowercase()
        .ok_if(|value| !value.is_empty())
        .ok_or_else(|| AppError::invalid_request("email is required"))?;

    let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
    db.remember_otp(&email, &code, &otp_cfg.pepper, otp_cfg.ttl_seconds)
        .await?;

    mail.send(
        &email,
        "Your WorshipViewer login code",
        &format!("Hello {email},\n\nto complete your sign-in or verification for WorshipViewer, please use the one-time password below:\n\n🔐 OTP: {code}\n\nThis code is valid for the next 5 minutes.  \nIf you did not request this, please ignore this message.\n\nBlessings,\nThe WorshipViewer Team"),
    ).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/auth/otp/verify",
    request_body = OtpVerify,
    responses(
        (status = 200, description = "OTP verified successfully; session cookie issued. When `WORSHIP_OTP_ALLOW_SELF_SIGNUP` is unset/true, a new user may be created for an unknown email; when false, the email must already exist. Response always embeds full `User` under `user`.", body = SessionBody),
        (status = 400, description = "OTP verification failed, or signup disabled for unknown email", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Too many incorrect attempts; request a new code", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to create session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Auth"
)]
#[post("/otp/verify")]
async fn otp_verify(
    db: Data<Database>,
    user_svc: Data<UserServiceHandle>,
    session_svc: Data<SessionServiceHandle>,
    cookie_cfg: Data<CookieConfig>,
    otp_cfg: Data<OtpConfig>,
    payload: web::Json<OtpVerify>,
) -> Result<HttpResponse, AppError> {
    let email = payload
        .email
        .trim()
        .to_lowercase()
        .ok_if(|value| !value.is_empty())
        .ok_or_else(|| AppError::invalid_request("email is required"))?;

    let code = payload
        .code
        .trim()
        .to_string()
        .ok_if(|value| !value.is_empty())
        .ok_or_else(|| AppError::invalid_request("otp code is required"))?;

    db.validate_otp(&email, &code, &otp_cfg.pepper, otp_cfg.max_attempts)
        .await?;

    let user = if otp_cfg.allow_self_signup {
        user_svc.get_user_by_email_or_create(&email).await?
    } else {
        user_svc.get_user_by_email(&email).await?.ok_or_else(|| {
            AppError::invalid_request(
                "no user exists for this email; self-signup via OTP is disabled",
            )
        })?
    };
    let session = session_svc
        .create_session(Session::new(user, cookie_cfg.session_ttl_seconds as i64))
        .await?;

    Ok(HttpResponse::Ok()
        .cookie(session_cookie(&session.id, &cookie_cfg))
        .json(SessionBody::from_session(session, true)))
}

trait OkIf {
    fn ok_if<F: FnOnce(&Self) -> bool>(self, condition: F) -> Option<Self>
    where
        Self: Sized;
}

impl OkIf for String {
    fn ok_if<F: FnOnce(&Self) -> bool>(self, condition: F) -> Option<Self> {
        if condition(&self) { Some(self) } else { None }
    }
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
