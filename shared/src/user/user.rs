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
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
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
            read: vec![],
            write: vec![],
            default_collection: None,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }

    #[cfg(feature = "backend")]
    pub fn read(&self) -> Vec<String> {
        self.read
            .iter()
            .cloned()
            .chain(std::iter::once(self.id.clone()))
            .chain(std::iter::once("public".to_string()))
            .collect()
    }

    #[cfg(feature = "backend")]
    pub fn write(&self) -> Vec<String> {
        self.write
            .iter()
            .cloned()
            .chain(std::iter::once(self.id.clone()))
            .collect()
    }
}
