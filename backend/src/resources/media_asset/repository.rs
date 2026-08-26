use async_trait::async_trait;
use surrealdb::types::RecordId;

use shared::MediaAsset;

use crate::error::AppError;

#[async_trait]
pub trait MediaAssetRepository: Send + Sync {
    async fn create_staging(
        &self,
        payload: super::model::CreateStagingAsset,
    ) -> Result<MediaAsset, AppError>;

    async fn get_asset(
        &self,
        read_teams: &[RecordId],
        asset_id: &str,
    ) -> Result<MediaAsset, AppError>;

    async fn get_staging_by_operation(&self, operation_id: &str) -> Result<MediaAsset, AppError>;

    async fn promote_staging(
        &self,
        asset_id: &str,
        operation_id: &str,
        etag: String,
    ) -> Result<MediaAsset, AppError>;

    async fn delete_asset(&self, asset_id: &str) -> Result<(), AppError>;

    async fn delete_assets_for_media(&self, media_id: &str) -> Result<Vec<MediaAsset>, AppError>;

    async fn list_staging_older_than(
        &self,
        max_age_seconds: u64,
    ) -> Result<Vec<MediaAsset>, AppError>;

    async fn create_final(
        &self,
        asset_id: &str,
        payload: super::model::CreateFinalAsset,
    ) -> Result<MediaAsset, AppError>;

    async fn list_assets_for_media(&self, media_id: &str) -> Result<Vec<MediaAsset>, AppError>;

    async fn update_owner_for_media(&self, media_id: &str, owner: RecordId)
    -> Result<(), AppError>;
}
