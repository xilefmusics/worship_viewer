use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::resources::User;
use crate::settings::Settings;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct Session {
    pub id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user: User) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user,
            created_at: Utc::now().into(),
            expires_at: Utc::now()
                + chrono::Duration::seconds(Settings::global().session_ttl_seconds as i64),
        }
    }

    pub fn admin(user: User) -> Self {
        let mut session = Self::new(user);
        session.id = "admin".into();
        session
    }
}
