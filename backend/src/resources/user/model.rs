use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use super::{Role, User};
use crate::error::AppError;

pub fn user_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "user" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid user id"));
    }
    Ok(("user".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRecord {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    email: String,
    #[serde(default)]
    role: Role,
    #[serde(default)]
    default_collection: Option<Thing>,
    created_at: Datetime,
    #[serde(default)]
    last_login_at: Option<Datetime>,
    #[serde(default)]
    request_count: u64,
}

impl UserRecord {
    pub fn into_user(self) -> User {
        User {
            id: self.id.unwrap().id.to_string(),
            email: self.email,
            role: self.role,
            default_collection: self.default_collection.map(|id| id.id.to_string()),
            created_at: self.created_at.into(),
            last_login_at: self.last_login_at.map(Into::into),
            request_count: self.request_count,
        }
    }

    pub fn from_user(user: User) -> Self {
        Self {
            id: if !user.id.is_empty() {
                Some(Thing::from(("user".to_owned(), user.id)))
            } else {
                None
            },
            email: user.email,
            role: user.role,
            default_collection: user
                .default_collection
                .map(|id| Thing::from(("collection".to_owned(), id))),
            created_at: user.created_at.into(),
            last_login_at: user.last_login_at.map(Into::into),
            request_count: user.request_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    /// Plain id returns ("user", id).
    #[test]
    fn user_resource_plain_id_ok() {
        let result = user_resource("some-uuid").unwrap();
        assert_eq!(result, ("user".to_owned(), "some-uuid".to_owned()));
    }

    /// "user:someid" Thing string is parsed correctly.
    #[test]
    fn user_resource_thing_string_ok() {
        let result = user_resource("user:someid").unwrap();
        assert_eq!(result.0, "user");
        assert_eq!(result.1, "someid");
    }

    /// BLC-HTTP-001: "team:abc" (wrong table) returns an error.
    #[test]
    fn blc_http_001_user_resource_wrong_table_err() {
        let err = user_resource("team:abc").unwrap_err();
        assert!(matches!(err, AppError::InvalidRequest(_)));
    }
}
