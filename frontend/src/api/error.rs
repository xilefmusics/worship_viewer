use shared::error::ErrorResponse;
use crate::components::toast_notifications::{show_warning, show_error};

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    InternalServerError(String),
    Network(String),
}

#[derive(Clone, Copy)]
pub enum OperationType {
    Read,
    Write,
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

    pub fn from_error_response(status: u16, payload: ErrorResponse) -> Self {
        Self::new(status, payload.error)
    }

    pub fn check_and_notify_offline(operation_type: OperationType) -> bool {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            // Use js_sys to access the onLine property
            let on_line = js_sys::Reflect::get(&navigator, &"onLine".into())
                .ok()
                .and_then(|val| val.as_bool())
                .unwrap_or(true);
            
            if !on_line {
                match operation_type {
                    OperationType::Read => {
                        show_warning("Offline", "You are currently offline. Some data may not be available.");
                    }
                    OperationType::Write => {
                        show_error("Offline", "Cannot modify data while offline. Please check your connection.");
                    }
                }
                return true;
            }
        }
        false
    }
}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        // Don't show notification here - it's already shown before the request
        Self::Network(err.to_string())
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
