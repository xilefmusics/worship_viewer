#[cfg(feature = "backend")]
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::Role;
#[cfg(feature = "backend")]
use super::User;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub email: String,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

impl CreateUserRequest {
    #[cfg(feature = "backend")]
    pub fn into_user(self) -> User {
        User {
            id: String::new(),
            email: self.email,
            role: self.role,
            read: self.read,
            write: self.write,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }
}

