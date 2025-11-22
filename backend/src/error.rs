use actix_web::http::StatusCode;
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
    #[error("internal error: {0}")]
    Internal(String),
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
                    AppError::invalid_request(dberr.to_string())
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

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if matches!(self, AppError::Internal(_)) {
            error!("{}", self);
        }
        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
}
