use actix_web::http::StatusCode;
use actix_web::web::JsonConfig;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("too many requests: {0}")]
    TooManyRequests(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Return an actix-web `JsonConfig` that maps deserialization errors
/// (including unknown fields from `deny_unknown_fields`) to a well-formed
/// 400 `AppError::InvalidRequest` response instead of the default plain-text
/// 400 from actix-web.
pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _req| {
        let message = err.to_string();
        actix_web::error::InternalError::from_response(
            err,
            AppError::InvalidRequest(message).error_response(),
        )
        .into()
    })
}

impl AppError {
    pub fn unauthorized() -> Self {
        Self::Unauthorized
    }

    pub fn forbidden() -> Self {
        Self::Forbidden
    }

    pub fn invalid_request<T: Into<String>>(msg: T) -> Self {
        Self::InvalidRequest(msg.into())
    }

    pub fn invalid_state() -> Self {
        Self::InvalidRequest("login state missing or expired".into())
    }

    pub fn oidc<E: std::fmt::Display>(err: E) -> Self {
        Self::Internal(err.to_string())
    }

    pub fn database<E: std::fmt::Display>(err: E) -> Self {
        Self::Internal(format!("database error: {}", err))
    }

    pub fn mail<E: std::fmt::Display>(err: E) -> Self {
        Self::Internal(format!("mail error: {}", err))
    }

    pub fn conflict<T: Into<String>>(msg: T) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn too_many_requests<T: Into<String>>(msg: T) -> Self {
        Self::TooManyRequests(msg.into())
    }
}

impl From<surrealdb::Error> for AppError {
    fn from(err: surrealdb::Error) -> Self {
        if let surrealdb::Error::Db(dberr) = &err {
            match dberr {
                surrealdb::error::Db::IndexExists { .. }
                | surrealdb::error::Db::RecordExists { .. }
                | surrealdb::error::Db::TxKeyAlreadyExists
                | surrealdb::error::Db::TxConditionNotMet => Self::conflict(dberr.to_string()),
                surrealdb::error::Db::FieldCheck { .. }
                | surrealdb::error::Db::FieldValue { .. }
                | surrealdb::error::Db::InvalidField { .. }
                | surrealdb::error::Db::InvalidArguments { .. }
                | surrealdb::error::Db::InvalidParam { .. }
                | surrealdb::error::Db::InvalidPatch { .. }
                | surrealdb::error::Db::InvalidQuery { .. }
                | surrealdb::error::Db::IdInvalid { .. }
                | surrealdb::error::Db::InvalidUrl { .. }
                | surrealdb::error::Db::SetCheck { .. }
                | surrealdb::error::Db::TableCheck { .. } => {
                    // Log the raw DB error (contains internal field/index names) and
                    // return a sanitized message so database internals are not leaked
                    // to clients.
                    error!("database validation error: {dberr}");
                    AppError::invalid_request("invalid request")
                }
                surrealdb::error::Db::NoRecordFound | surrealdb::error::Db::IdNotFound { .. } => {
                    AppError::NotFound("record not found".into())
                }
                _ => Self::database(err),
            }
        } else {
            Self::database(err)
        }
    }
}

impl From<chordlib::Error> for AppError {
    fn from(err: chordlib::Error) -> Self {
        AppError::invalid_request(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Internal(format!("HTTP client error: {}", err))
    }
}

impl AppError {
    /// Stable machine-readable code for this error variant.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidRequest(_) => "invalid_request",
            AppError::Conflict(_) => "conflict",
            AppError::TooManyRequests(_) => "too_many_requests",
            AppError::Internal(_) => "internal",
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if matches!(self, AppError::Internal(_)) {
            error!("{}", self);
        }
        // For Internal errors, strip the internal detail from the client response.
        let message = if matches!(self, AppError::Internal(_)) {
            "internal server error".to_owned()
        } else {
            self.to_string()
        };
        HttpResponse::build(self.status_code())
            .json(json!({ "code": self.code(), "error": message }))
    }
}
