use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use awc::error::{PayloadError, SendRequestError};
use fancy_surreal;
use shared::ChordlibError;
use std::error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Unauthorized(String),
    Filesystem(String),
    NotFound(String),
    SendRequest(awc::error::SendRequestError),
    Payload(awc::error::PayloadError),
    ChordlibError(String),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(message) => write!(f, "DatabaseError ({})", message),
            Self::Unauthorized(message) => write!(f, "UnauthorizedError ({})", message),
            Self::Filesystem(message) => write!(f, "FilesystemError ({})", message),
            Self::NotFound(message) => write!(f, "NotFoundError ({})", message),
            Self::ChordlibError(message) => write!(f, "ChordlibError ({})", message),
            Self::SendRequest(err) => write!(f, "SendRequest ({})", err.to_string()),
            Self::Payload(err) => write!(f, "Payload ({})", err.to_string()),
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

impl From<SendRequestError> for AppError {
    fn from(err: SendRequestError) -> Self {
        Self::SendRequest(err)
    }
}

impl From<PayloadError> for AppError {
    fn from(err: PayloadError) -> Self {
        Self::Payload(err)
    }
}

impl From<ChordlibError> for AppError {
    fn from(err: ChordlibError) -> Self {
        Self::ChordlibError(err.to_string())
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(err: FromUtf8Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Filesystem(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ChordlibError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SendRequest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Payload(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            Self::ChordlibError(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
            Self::SendRequest(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
            Self::Payload(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
            Self::NotFound(_) => HttpResponse::build(self.status_code()).body("404 Not Found"),
            Self::Other(_) => {
                HttpResponse::build(self.status_code()).body("500 Internal Server Error")
            }
        }
    }
}
