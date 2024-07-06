use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use fancy_surreal;
use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Unauthorized(String),
    Filesystem(String),
    NotFound(String),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(message) => write!(f, "DatabaseError ({})", message),
            Self::Unauthorized(message) => write!(f, "UnauthorizedError ({})", message),
            Self::Filesystem(message) => write!(f, "FilesystemError ({})", message),
            Self::NotFound(message) => write!(f, "NotFoundError ({})", message),
            Self::Other(message) => write!(f, "OtherError ({})", message),
        }
    }
}

impl error::Error for AppError {}

impl From<fancy_surreal::Error> for AppError {
    fn from(err: fancy_surreal::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        Self::Filesystem(err.to_string())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Filesystem(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self);
        match self {
            Self::Database(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
            Self::Unauthorized(_) => {
                HttpResponse::build(self.status_code()).body("401 Unauthorized")
            }
            Self::Filesystem(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
            Self::NotFound(_) => HttpResponse::build(self.status_code()).body("404 Not Found"),
            Self::Other(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
        }
    }
}
