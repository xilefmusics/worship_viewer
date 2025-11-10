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

#[derive(Clone, Debug, Deserialize, Default)]
pub struct User {
    #[allow(unused)]
    pub id: String,
    #[allow(unused)]
    pub email: String,
    #[allow(unused)]
    pub role: Role,
    #[allow(unused)]
    pub read: Vec<String>,
    #[allow(unused)]
    pub write: Vec<String>,
    #[allow(unused)]
    pub last_login_at: Option<DateTime<Utc>>,
    #[allow(unused)]
    pub request_count: u64,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Session {
    #[allow(unused)]
    pub id: String,
    #[allow(unused)]
    pub user: User,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
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
