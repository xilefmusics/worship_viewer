use async_trait::async_trait;
use surrealdb::types::RecordId;

use shared::api::ListQuery;
use shared::media::Media;

use crate::error::AppError;

use super::model::MediaWrite;

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn list(&self, read_teams: &[RecordId], query: ListQuery)
    -> Result<Vec<Media>, AppError>;
    async fn count(&self, read_teams: &[RecordId], q: Option<&str>) -> Result<u64, AppError>;
    async fn get(&self, read_teams: &[RecordId], id: &str) -> Result<Media, AppError>;
    async fn create(&self, owner: RecordId, value: MediaWrite) -> Result<Media, AppError>;
    async fn create_with_id(
        &self,
        id: &str,
        owner: RecordId,
        value: MediaWrite,
    ) -> Result<Media, AppError>;
    async fn update(
        &self,
        write_teams: &[RecordId],
        id: &str,
        owner: Option<RecordId>,
        value: MediaWrite,
    ) -> Result<Media, AppError>;
    async fn move_owner(
        &self,
        write_teams: &[RecordId],
        id: &str,
        owner: RecordId,
    ) -> Result<Media, AppError>;
    async fn delete(&self, write_teams: &[RecordId], id: &str) -> Result<Media, AppError>;
    /// Internal read without ACL checks (processing jobs and reconciliation).
    async fn get_unscoped(&self, id: &str) -> Result<Media, AppError>;
    async fn update_unscoped(&self, id: &str, value: MediaWrite) -> Result<Media, AppError>;
}
