use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use shared::api::ListQuery;
use shared::blob::{Blob, CreateBlob};

use crate::database::Database;
use crate::error::AppError;
use crate::settings::Settings;

pub trait Model {
    async fn get_blobs(
        &self,
        owners: Vec<String>,
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError>;
    async fn get_blob(&self, owners: Vec<String>, id: &str) -> Result<Blob, AppError>;
    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError>;
    async fn update_blob(
        &self,
        owners: Vec<String>,
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError>;
    async fn delete_blob(&self, owners: Vec<String>, id: &str) -> Result<Blob, AppError>;
}

impl Model for Database {
    async fn get_blobs(
        &self,
        owners: Vec<String>,
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError> {
        let owners = owners
            .into_iter()
            .map(|owner_id| owner_thing(&owner_id))
            .collect::<Vec<_>>();

        let mut query = String::from("SELECT * FROM blob WHERE owner IN $owners");
        if pagination.to_offset_limit().is_some() {
            query.push_str(" LIMIT $limit START $start");
        }

        let mut request = self.db.query(query).bind(("owners", owners));
        if let Some((offset, limit)) = pagination.to_offset_limit() {
            request = request.bind(("limit", limit)).bind(("start", offset));
        }

        let mut response = request.await.map_err(AppError::database)?;

        Ok(response
            .take::<Vec<BlobRecord>>(0)?
            .into_iter()
            .map(BlobRecord::into_blob)
            .collect())
    }

    async fn get_blob(&self, owners: Vec<String>, id: &str) -> Result<Blob, AppError> {
        match self.db.select(blob_resource(id)?).await? {
            Some(record) if blob_belongs_to(&record, owners) => Ok(record.into_blob()),
            _ => Err(AppError::NotFound("blob not found".into())),
        }
    }

    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError> {
        let created = self
            .db
            .create("blob")
            .content(BlobRecord::from_payload(
                None,
                Some(owner_thing(owner)),
                Some(Utc::now().into()),
                blob,
            ))
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::database("failed to create blob"))?;
        write_blob_file(&created)?;
        Ok(created)
    }

    async fn update_blob(
        &self,
        owners: Vec<String>,
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError> {
        let resource = blob_resource(id)?;
        let existing = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("blob not found".into()))?;

        if !blob_belongs_to(&existing, owners) {
            return Err(AppError::NotFound("blob not found".into()));
        }

        let record_id = Thing::from(resource.clone());
        let record = BlobRecord::from_payload(
            Some(record_id),
            existing.owner.clone(),
            existing.created_at.clone(),
            blob,
        );

        let updated = self
            .db
            .update(resource)
            .content(record)
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::database("failed to update blob"))?;
        write_blob_file(&updated)?;
        Ok(updated)
    }

    async fn delete_blob(&self, owner: Vec<String>, id: &str) -> Result<Blob, AppError> {
        let resource = blob_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !blob_belongs_to(&existing, owner) {
                return Err(AppError::NotFound("blob not found".into()));
            }
        } else {
            return Err(AppError::NotFound("blob not found".into()));
        }

        let deleted = self
            .db
            .delete(resource)
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))?;
        if let Some(name) = deleted.file_name() {
            let path = Path::new(&Settings::global().blob_dir).join(name);
            let _ = std::fs::remove_file(path);
        }
        Ok(deleted)
    }
}

fn write_blob_file(blob: &Blob) -> Result<(), AppError> {
    let file_name = blob
        .file_name()
        .ok_or_else(|| AppError::Internal("blob has no id".into()))?;
    let path = Path::new(&Settings::global().blob_dir).join(file_name);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| AppError::Internal(e.to_string()))?;
    }
    std::fs::write(&path, []).map_err(|e| AppError::Internal(e.to_string()))
}

fn blob_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "blob" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid blob id"));
    }

    Ok(("blob".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BlobRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    file_type: shared::blob::FileType,
    width: u32,
    height: u32,
    #[serde(default)]
    ocr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<Datetime>,
}

impl BlobRecord {
    fn into_blob(self) -> Blob {
        Blob {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            file_type: self.file_type,
            width: self.width,
            height: self.height,
            ocr: self.ocr,
        }
    }

    fn from_payload(
        id: Option<Thing>,
        owner: Option<Thing>,
        created_at: Option<Datetime>,
        blob: CreateBlob,
    ) -> Self {
        Self {
            id,
            owner,
            file_type: blob.file_type,
            width: blob.width,
            height: blob.height,
            ocr: blob.ocr,
            created_at,
        }
    }
}

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

fn blob_belongs_to(record: &BlobRecord, owners: Vec<String>) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owners.contains(&owner.id.to_string()))
        .unwrap_or(false)
}
