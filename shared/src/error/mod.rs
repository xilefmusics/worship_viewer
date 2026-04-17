use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Standard error envelope returned by all API error responses.
///
/// `code` is a stable, machine-readable identifier; `error` is a human-readable
/// description.
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    /// Stable machine-readable error code, e.g. `"unauthorized"`, `"not_found"`.
    pub code: String,
    /// Human-readable description of the error.
    pub error: String,
}

/// [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) problem document (`application/problem+json`).
///
/// Includes `code` and `error` alongside `detail` so older clients that only read `error` keep working.
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProblemDetails {
    /// URI reference identifying the problem type; `about:blank` when omitted in responses.
    #[serde(rename = "type")]
    pub type_uri: String,
    /// Short, stable summary of the problem class.
    pub title: String,
    /// HTTP status code.
    pub status: u16,
    /// Human-readable explanation (same value as [`Self::error`]).
    pub detail: String,
    /// Optional URI reference that identifies the specific occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Stable machine-readable code (extension member).
    pub code: String,
    /// Same as `detail` (legacy alias).
    pub error: String,
}

#[derive(Clone, Debug, Error)]
pub enum NetworkClientError {
    #[error("request failed (status: {status:?}): {message}")]
    RequestFailed {
        status: Option<u16>,
        message: String,
    },

    #[error("connection error")]
    Connection,

    #[error("serialization error: {message}")]
    Serialization { message: String },

    #[error("invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("unexpected error: {message}")]
    Unexpected { message: String },
}

impl From<serde_json::Error> for NetworkClientError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "cli")]
impl From<reqwest::Error> for NetworkClientError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() || err.is_connect() {
            return Self::Connection;
        }

        if let Some(status) = err.status() {
            return Self::RequestFailed {
                status: Some(status.as_u16()),
                message: err.to_string(),
            };
        }

        Self::Unexpected {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<gloo_net::Error> for NetworkClientError {
    fn from(err: gloo_net::Error) -> Self {
        Self::Unexpected {
            message: err.to_string(),
        }
    }
}
