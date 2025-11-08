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
