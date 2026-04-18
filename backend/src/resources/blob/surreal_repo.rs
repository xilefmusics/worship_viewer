use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use surrealdb::sql::Thing;

use serde::Deserialize;

use shared::api::ListQuery;
use shared::blob::{Blob, CreateBlob};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::{belongs_to, resource_id};

use super::model::BlobRecord;
use super::repository::BlobRepository;

#[derive(Clone)]
pub struct SurrealBlobRepo {
    db: Arc<Database>,
}

impl SurrealBlobRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

fn blob_list_q_needle(q: &ListQuery) -> Option<String> {
    q.q.as_ref().and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_lowercase())
        }
    })
}

#[async_trait]
impl BlobRepository for SurrealBlobRepo {
    async fn get_blobs(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError> {
        let db = self.inner();
        let (offset, limit) = pagination.effective_offset_limit();
        let needle = blob_list_q_needle(&pagination);

        let mut response = if let Some(needle) = needle {
            db.db
                .query(
                    "SELECT * FROM blob WHERE owner IN $teams AND \
                     string::contains(string::lowercase(ocr), $needle) LIMIT $limit START $start",
                )
                .bind(("teams", read_teams.to_vec()))
                .bind(("needle", needle))
                .bind(("limit", limit))
                .bind(("start", offset))
                .await
                .map_err(AppError::database)?
        } else {
            db.db
                .query("SELECT * FROM blob WHERE owner IN $teams LIMIT $limit START $start")
                .bind(("teams", read_teams.to_vec()))
                .bind(("limit", limit))
                .bind(("start", offset))
                .await
                .map_err(AppError::database)?
        };

        Ok(response
            .take::<Vec<BlobRecord>>(0)?
            .into_iter()
            .map(BlobRecord::into_blob)
            .collect())
    }

    async fn count_blobs(
        &self,
        read_teams: &[Thing],
        pagination: &ListQuery,
    ) -> Result<u64, AppError> {
        #[derive(Deserialize)]
        struct CountResult {
            count: u64,
        }
        let needle = blob_list_q_needle(pagination);
        let mut response = if let Some(needle) = needle {
            self.inner()
                .db
                .query(
                    "SELECT count() FROM blob WHERE owner IN $teams AND \
                     string::contains(string::lowercase(ocr), $needle) GROUP ALL",
                )
                .bind(("teams", read_teams.to_vec()))
                .bind(("needle", needle))
                .await?
        } else {
            self.inner()
                .db
                .query("SELECT count() FROM blob WHERE owner IN $teams GROUP ALL")
                .bind(("teams", read_teams.to_vec()))
                .await?
        };
        Ok(response
            .take::<Vec<CountResult>>(0)?
            .into_iter()
            .next()
            .map(|r| r.count)
            .unwrap_or(0))
    }

    async fn get_blob(&self, read_teams: &[Thing], id: &str) -> Result<Blob, AppError> {
        let db = self.inner();
        let record: Option<BlobRecord> = db.db.select(resource_id("blob", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, read_teams) => Ok(r.into_blob()),
            _ => Err(AppError::NotFound("blob not found".into())),
        }
    }

    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError> {
        let db = self.inner();
        let owner_team = db.personal_team_thing_for_user(owner).await?;
        db.db
            .create("blob")
            .content(BlobRecord::from_payload(
                None,
                Some(owner_team),
                Some(Utc::now().into()),
                blob,
            ))
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::database("failed to create blob"))
    }

    async fn update_blob(
        &self,
        write_teams: &[Thing],
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("blob", id)?;

        let mut response = db
            .db
            .query(
                "UPDATE type::thing($tb, $sid) SET file_type = $file_type, width = $width, \
                 height = $height, ocr = $ocr WHERE owner IN $teams RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("file_type", blob.file_type))
            .bind(("width", blob.width))
            .bind(("height", blob.height))
            .bind(("ocr", blob.ocr.clone()))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<BlobRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))
    }

    async fn delete_blob(&self, write_teams: &[Thing], id: &str) -> Result<Blob, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("blob", id)?;
        let mut response = db
            .db
            .query("DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE")
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<BlobRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))
    }
}
