use serde::Deserialize;

use shared::api::ListQuery;

use crate::database::Database;
use crate::error::AppError;

use super::model::{HttpAuditLog, HttpAuditRecord};

pub struct MonitoringRepo;

impl MonitoringRepo {
    pub async fn count_http_audit_logs(db: &Database) -> Result<u64, AppError> {
        #[derive(Deserialize)]
        struct CountResult {
            count: u64,
        }
        let mut response = db
            .db
            .query("SELECT count() FROM http_request_audit GROUP ALL")
            .await
            .map_err(|e| crate::log_and_convert!(AppError::database, "http_audit.count", e))?;
        Ok(response
            .take::<Vec<CountResult>>(0)
            .map_err(|e| crate::log_and_convert!(AppError::database, "http_audit.count.take", e))?
            .into_iter()
            .next()
            .map(|r| r.count)
            .unwrap_or(0))
    }

    pub async fn list_http_audit_logs(
        db: &Database,
        query: ListQuery,
    ) -> Result<Vec<HttpAuditLog>, AppError> {
        let (offset, limit) = query.effective_offset_limit();
        let mut response = db
            .db
            .query(
                "SELECT * FROM http_request_audit ORDER BY created_at DESC LIMIT $limit START $start",
            )
            .bind(("limit", limit))
            .bind(("start", offset))
            .await
            .map_err(|e| crate::log_and_convert!(AppError::database, "http_audit.list", e))?;
        let rows: Vec<HttpAuditRecord> = response
            .take(0)
            .map_err(|e| crate::log_and_convert!(AppError::database, "http_audit.list.take", e))?;
        Ok(rows.into_iter().map(|r| r.into_wire()).collect())
    }
}
