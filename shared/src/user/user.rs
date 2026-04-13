use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Role;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct User {
    pub id: String,
    pub email: String,
    pub role: Role,
    #[serde(default)]
    pub default_collection: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub last_login_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub request_count: u64,
}

impl User {
    #[cfg(feature = "backend")]
    pub fn new<S: Into<String>>(email: S) -> Self {
        Self {
            id: String::new(),
            email: email.into().to_lowercase(),
            role: Role::default(),
            default_collection: None,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }
}
