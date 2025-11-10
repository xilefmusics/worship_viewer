#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    InternalServerError(String),
    Network(String),
}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl ApiError {
    pub fn new(status: u16, msg: String) -> Self {
        match status {
            400 => Self::BadRequest(msg),
            401 => Self::Unauthorized(msg),
            403 => Self::Forbidden(msg),
            409 => Self::Conflict(msg),
            500 => Self::InternalServerError(msg),
            _ => Self::Network(msg),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            ApiError::Conflict(msg) => write!(f, "Conflict: {msg}"),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {msg}"),
            ApiError::Network(msg) => write!(f, "Network Error: {msg}"),
        }
    }
}
