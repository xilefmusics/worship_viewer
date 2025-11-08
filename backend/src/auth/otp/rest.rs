use actix_web::{
    HttpResponse,
    cookie::{Cookie, SameSite},
    post,
    web::{self, Data},
};

use rand::Rng;
use serde::Deserialize;
use time::Duration as CookieDuration;
use utoipa::ToSchema;

use super::Model;
use crate::database::Database;
use crate::error::AppError;
use crate::mail::Mail;
use crate::resources::{Session, SessionModel, UserModel};
use crate::settings::Settings;

#[utoipa::path(
    post,
    path = "/auth/otp/request",
    request_body = OtpRequest,
    responses(
        (status = 204, description = "OTP generated and delivered out-of-band"),
        (status = 400, description = "Email missing or invalid", body = ErrorResponse),
        (status = 500, description = "Failed to persist or deliver OTP", body = ErrorResponse)
    ),
    tag = "Auth"
)]
#[post("/otp/request")]
async fn otp_request(
    db: Data<Database>,
    payload: web::Json<OtpRequest>,
) -> Result<HttpResponse, AppError> {
    let email = payload
        .email
        .trim()
        .to_lowercase()
        .ok_if(|value| !value.is_empty())
        .ok_or_else(|| AppError::invalid_request("email is required"))?;

    let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
    db.remember_otp(&email, &code).await?;

    Mail::default()
        .to(&email)
        .subject("Your WorshipViewer login code")
        .body(&format!("Hello {email},\n\nto complete your sign-in or verification for WorshipViewer, please use the one-time password below:\n\n🔐 OTP: {code}\n\nThis code is valid for the next 5 minutes.  \nIf you did not request this, please ignore this message.\n\nBlessings,\nThe WorshipViewer Team"))
        .send()?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/auth/otp/verify",
    request_body = OtpVerify,
    responses(
        (status = 200, description = "OTP verified successfully; session cookie issued", body = Session),
        (status = 400, description = "OTP verification failed", body = ErrorResponse),
        (status = 500, description = "Failed to create session", body = ErrorResponse)
    ),
    tag = "Auth"
)]
#[post("/otp/verify")]
async fn otp_verify(
    db: Data<Database>,
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

    db.validate_otp(&email, &code).await?;

    let session = db
        .create_session(Session::new(db.get_user_by_email_or_create(&email).await?))
        .await?;

    Ok(HttpResponse::Ok()
        .cookie(session_cookie(&session.id))
        .json(session))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OtpRequest {
    email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OtpVerify {
    email: String,
    code: String,
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

fn session_cookie(session_id: &str) -> Cookie<'static> {
    let settings = Settings::global();
    let mut builder = Cookie::build(settings.cookie_name.clone(), session_id.to_owned())
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(settings.cookie_secure);

    if settings.session_ttl_seconds > 0 {
        builder = builder.max_age(CookieDuration::seconds(settings.session_ttl_seconds as i64));
    }

    builder.finish()
}
