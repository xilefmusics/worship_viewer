use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use surrealdb::types::RecordId;

use shared::MediaAsset;

use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::{belongs_to, resource_id};

use super::model::{CreateStagingAsset, MediaAssetRecord};
use super::repository::MediaAssetRepository;

#[derive(Clone)]
pub struct SurrealMediaAssetRepo {
    db: Arc<Database>,
}

impl SurrealMediaAssetRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MediaAssetRepository for SurrealMediaAssetRepo {
    async fn create_staging(&self, payload: CreateStagingAsset) -> Result<MediaAsset, AppError> {
        let record = MediaAssetRecord::from_staging(
            payload.owner,
            payload.media_id,
            payload.kind,
            payload.content_type,
            payload.byte_length,
            payload.operation_id,
            payload.etag,
        );
        let created: Option<MediaAssetRecord> =
            self.db.db.create("media_asset").content(record).await?;
        created
            .ok_or_else(|| AppError::database("failed to create media asset"))?
            .into_asset()
    }

    async fn get_asset(
        &self,
        read_teams: &[RecordId],
        asset_id: &str,
    ) -> Result<MediaAsset, AppError> {
        let row: Option<MediaAssetRecord> = self
            .db
            .db
            .select(resource_id("media_asset", asset_id)?)
            .await?;
        match row {
            Some(row) if belongs_to(&row.owner, read_teams) => row.into_asset(),
            _ => Err(AppError::NotFound("media asset not found".into())),
        }
    }

    async fn get_staging_by_operation(&self, operation_id: &str) -> Result<MediaAsset, AppError> {
        let mut response = self
            .db
            .db
            .query(
                "SELECT * FROM media_asset WHERE operation_id = $op AND status = 'staging' LIMIT 1",
            )
            .bind(("op", operation_id.to_owned()))
            .await?;
        response
            .take::<Vec<MediaAssetRecord>>(0)?
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("staging upload not found".into()))?
            .into_asset()
    }

    async fn promote_staging(
        &self,
        asset_id: &str,
        operation_id: &str,
        etag: String,
    ) -> Result<MediaAsset, AppError> {
        let (tb, sid) = resource_id("media_asset", asset_id)?;
        let mut response = self
            .db
            .db
            .query(
                "UPDATE type::record($tb, $sid) SET status = 'final', operation_id = NONE, etag = $etag WHERE operation_id = $op AND status = 'staging' RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("etag", etag))
            .bind(("op", operation_id.to_owned()))
            .await?;
        response
            .take::<Vec<MediaAssetRecord>>(0)?
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("staging upload not found".into()))?
            .into_asset()
    }

    async fn delete_asset(&self, asset_id: &str) -> Result<(), AppError> {
        let rid = resource_id("media_asset", asset_id)?;
        let _: Option<MediaAssetRecord> = self.db.db.delete(rid).await?;
        Ok(())
    }

    async fn delete_assets_for_media(&self, media_id: &str) -> Result<Vec<MediaAsset>, AppError> {
        let (tb, sid) = resource_id("media", media_id)?;
        let mut response = self
            .db
            .db
            .query("DELETE media_asset WHERE media_id = type::record($tb, $sid) RETURN BEFORE")
            .bind(("tb", tb))
            .bind(("sid", sid))
            .await?;
        response
            .take::<Vec<MediaAssetRecord>>(0)?
            .into_iter()
            .map(MediaAssetRecord::into_asset)
            .collect()
    }

    async fn list_staging_older_than(
        &self,
        max_age_seconds: u64,
    ) -> Result<Vec<MediaAsset>, AppError> {
        let cutoff = Utc::now() - Duration::seconds(max_age_seconds as i64);
        let cutoff_dt: surrealdb::types::Datetime = cutoff.into();
        let mut response = self
            .db
            .db
            .query("SELECT * FROM media_asset WHERE status = 'staging' AND created_at < $cutoff")
            .bind(("cutoff", cutoff_dt))
            .await?;
        response
            .take::<Vec<MediaAssetRecord>>(0)?
            .into_iter()
            .map(MediaAssetRecord::into_asset)
            .collect()
    }
}
