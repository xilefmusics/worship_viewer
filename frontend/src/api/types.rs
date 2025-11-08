use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ApiError;

#[derive(Serialize)]
pub struct OtpRequestPayload {
    pub email: String,
}

#[derive(Serialize)]
pub struct OtpVerifyPayload {
    pub email: String,
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    #[default]
    Default,
    Admin,
}

impl Role {
    pub fn to_str(&self) -> &'static str {
        match self {
            Role::Default => "default",
            Role::Admin => "admin",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct User {
    pub id: String,
    pub email: String,
    pub role: Role,
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub request_count: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    pub fn to_api_error(self, status: u16) -> ApiError {
        ApiError::new(status, self.error)
    }
}
