use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    #[default]
    Default,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub email: String,
    pub role: Role,
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub last_login_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub request_count: u64,
}

impl User {
    pub fn new<S: Into<String>>(email: S) -> Self {
        Self {
            id: String::new(),
            email: email.into().to_lowercase(),
            role: Role::default(),
            read: vec![],
            write: vec![],
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }
}
