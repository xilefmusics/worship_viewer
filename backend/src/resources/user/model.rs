use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, Kind, RecordId, SurrealValue, Value, kind};

use super::{Role, User};
use crate::database::record_id_string;
use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(transparent)]
struct RoleField(pub Role);

impl SurrealValue for RoleField {
    fn kind_of() -> Kind {
        kind!(any)
    }

    fn is_value(_value: &Value) -> bool {
        true
    }

    fn into_value(self) -> Value {
        let j = serde_json::to_value(self.0).unwrap_or(serde_json::Value::Null);
        j.into_value()
    }

    fn from_value(value: Value) -> surrealdb::Result<Self> {
        let j = serde_json::Value::from_value(value)?;
        serde_json::from_value(j)
            .map(RoleField)
            .map_err(|e| surrealdb::Error::internal(e.to_string()))
    }
}

pub fn user_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(rid) = RecordId::parse_simple(id) {
        if rid.table.as_str() == "user" {
            return Ok(("user".to_owned(), record_id_string(&rid)));
        }
        return Err(AppError::invalid_request("invalid user id"));
    }
    Ok(("user".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct UserRecord {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RecordId>,
    email: String,
    #[serde(default)]
    role: RoleField,
    #[serde(default)]
    default_collection: Option<RecordId>,
    created_at: Datetime,
    #[serde(default)]
    last_login_at: Option<Datetime>,
    #[serde(default)]
    request_count: u64,
}

impl UserRecord {
    pub fn into_user(self) -> User {
        User {
            id: self.id.map(|id| record_id_string(&id)).unwrap_or_default(),
            email: self.email,
            role: self.role.0,
            default_collection: self.default_collection.map(|id| record_id_string(&id)),
            created_at: self.created_at.into(),
            last_login_at: self.last_login_at.map(Into::into),
            request_count: self.request_count,
        }
    }

    pub fn from_user(user: User) -> Self {
        Self {
            id: if !user.id.is_empty() {
                Some(RecordId::new("user", user.id))
            } else {
                None
            },
            email: user.email,
            role: RoleField(user.role),
            default_collection: user
                .default_collection
                .map(|id| RecordId::new("collection", id)),
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

    /// "user:someid" record id string is parsed correctly.
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
