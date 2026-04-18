use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::blob::{Blob, CreateBlob};

use crate::error::AppError;

/// Pure blob data access (no user ACL — callers pass pre-resolved team [`Thing`]s).
#[async_trait]
pub trait BlobRepository: Send + Sync {
    async fn get_blobs(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError>;

    /// Count blobs visible to `read_teams`, applying the same optional `q` OCR substring filter as [`get_blobs`](Self::get_blobs).
    async fn count_blobs(
        &self,
        read_teams: &[Thing],
        pagination: &ListQuery,
    ) -> Result<u64, AppError>;

    async fn get_blob(&self, read_teams: &[Thing], id: &str) -> Result<Blob, AppError>;

    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError>;

    async fn update_blob(
        &self,
        write_teams: &[Thing],
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError>;

    async fn delete_blob(&self, write_teams: &[Thing], id: &str) -> Result<Blob, AppError>;
}
