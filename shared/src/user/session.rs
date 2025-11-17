use chrono::{DateTime, Utc};
#[cfg(feature = "backend")]
use chrono::Duration;
use serde::{Deserialize, Serialize};

use super::User;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Session {
    pub id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    #[cfg(feature = "backend")]
    pub fn new(user: User, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        let expires_at = if ttl_seconds > 0 {
            now + Duration::seconds(ttl_seconds)
        } else {
            now
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user,
            created_at: now,
            expires_at,
        }
    }

    #[cfg(feature = "backend")]
    pub fn admin(user: User, ttl_seconds: i64) -> Self {
        let mut session = Self::new(user, ttl_seconds);
        session.id = "admin".into();
        session
    }
}

