use chrono::{DateTime, Utc};
use serde::Serialize;
use surrealdb::sql::{Datetime, Thing};
use utoipa::ToSchema;

use crate::database::record_id_string;

#[derive(Debug, serde::Deserialize)]
pub struct HttpAuditRecord {
    #[serde(default)]
    pub id: Option<Thing>,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i64,
    pub duration_ms: i64,
    #[serde(default)]
    pub user: Option<Thing>,
    #[serde(default)]
    pub session: Option<Thing>,
    pub created_at: Datetime,
}

impl HttpAuditRecord {
    pub fn into_wire(self) -> HttpAuditLog {
        let id = self
            .id
            .as_ref()
            .map(record_id_string)
            .unwrap_or_default();
        HttpAuditLog {
            id,
            request_id: self.request_id,
            method: self.method,
            path: self.path,
            status_code: self.status_code as i32,
            duration_ms: self.duration_ms as i32,
            user_id: self.user.as_ref().map(record_id_string),
            session_id: self.session.as_ref().map(record_id_string),
            created_at: self.created_at.into(),
        }
    }
}

/// One persisted HTTP request audit row (admin monitoring API).
#[derive(Debug, Serialize, ToSchema)]
pub struct HttpAuditLog {
    pub id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub duration_ms: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
