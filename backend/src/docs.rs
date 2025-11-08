use serde::Serialize;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::auth::otp::rest::{OtpRequest, OtpVerify};
use crate::resources::user::Role;
use crate::resources::{CreateUserRequest, Session, User};

pub mod reset {
    use super::{ApiDoc, OpenApi};
    use utoipa_swagger_ui::SwaggerUi;

    pub fn scope() -> SwaggerUi {
        SwaggerUi::new("/api/docs/{_:.*}").url("/api/docs/openapi.json", ApiDoc::openapi())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::oidc::rest::login,
        crate::auth::oidc::rest::callback,
        crate::auth::otp::rest::otp_request,
        crate::auth::otp::rest::otp_verify,
        crate::auth::rest::logout,
        crate::resources::user::rest::get_user,
        crate::resources::user::rest::get_users,
        crate::resources::user::rest::get_user_by_id,
        crate::resources::user::rest::create_user,
        crate::resources::user::rest::delete_user,
        crate::resources::user::session::rest::get_sessions_for_current_user,
        crate::resources::user::session::rest::get_session_for_current_user,
        crate::resources::user::session::rest::delete_session_for_current_user,
        crate::resources::user::session::rest::get_sessions_for_user,
        crate::resources::user::session::rest::get_session_for_user,
        crate::resources::user::session::rest::create_session_for_user,
        crate::resources::user::session::rest::delete_session_for_user
    ),
    components(
        schemas(
            User,
            Session,
            Role,
            CreateUserRequest,
            OtpRequest,
            OtpVerify,
            ErrorResponse
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User resources")
    ),
    modifiers(&SessionSecurity)
)]
pub struct ApiDoc;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

struct SessionSecurity;

impl Modify for SessionSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "SessionCookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                "sso_session",
                "Session cookie returned after a successful authentication flow",
            ))),
        );
        components.add_security_scheme(
            "SessionToken",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "Authorization",
                "Optional session override using `Authorization` header (raw value or `Bearer <session>`)",
            ))),
        );
    }
}
