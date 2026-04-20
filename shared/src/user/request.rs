#[cfg(feature = "backend")]
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::Role;
#[cfg(feature = "backend")]
use super::User;
#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({ "email": "singer@example.com", "role": "default" }))
)]
pub struct CreateUser {
    pub email: String,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub default_collection: Option<String>,
}

impl CreateUser {
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
            oauth_picture_url: None,
            oauth_avatar_blob_id: None,
            avatar_blob_id: None,
        }
    }
}
