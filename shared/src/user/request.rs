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
    pub default_collection: Option<String>,
}

impl CreateUserRequest {
    #[cfg(feature = "backend")]
    pub fn into_user(self) -> User {
        User {
            id: String::new(),
            email: self.email,
            role: self.role,
            default_collection: self.default_collection,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }
}
