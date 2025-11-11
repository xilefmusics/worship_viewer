use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::blob::Blob;

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_blobs(&self) -> Result<Vec<Blob>, AppError>;
    async fn get_blob(&self, id: &str) -> Result<Blob, AppError>;
    async fn create_blob(&self, blob: Blob) -> Result<Blob, AppError>;
    async fn update_blob(&self, id: &str, blob: Blob) -> Result<Blob, AppError>;
    async fn delete_blob(&self, id: &str) -> Result<Blob, AppError>;
}

impl Model for Database {
    async fn get_blobs(&self) -> Result<Vec<Blob>, AppError> {
        Ok(self
            .db
            .select("blob")
            .await
            .map_err(AppError::database)?
            .into_iter()
            .map(BlobRecord::into_blob)
            .collect())
    }

    async fn get_blob(&self, id: &str) -> Result<Blob, AppError> {
        self.db
            .select(blob_resource(id)?)
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))
    }

    async fn create_blob(&self, blob: Blob) -> Result<Blob, AppError> {
        self.db
            .create("blob")
            .content(BlobRecord::from_blob(blob))
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::database("failed to create blob"))
    }

    async fn update_blob(&self, id: &str, blob: Blob) -> Result<Blob, AppError> {
        self.db
            .update(blob_resource(id)?)
            .content(BlobRecord::from_blob(blob))
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))
    }

    async fn delete_blob(&self, id: &str) -> Result<Blob, AppError> {
        self.db
            .delete(blob_resource(id)?)
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))
    }
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
    file_type: shared::blob::FileType,
    width: u32,
    height: u32,
    #[serde(default)]
    ocr: String,
}

impl BlobRecord {
    fn into_blob(self) -> Blob {
        Blob {
            id: self.id.map(|thing| thing.id.to_string()),
            file_type: self.file_type,
            width: self.width,
            height: self.height,
            ocr: self.ocr,
        }
    }

    fn from_blob(blob: Blob) -> Self {
        Self {
            id: blob
                .id
                .filter(|id| !id.is_empty())
                .map(|id| Thing::from(("blob".to_owned(), id))),
            file_type: blob.file_type,
            width: blob.width,
            height: blob.height,
            ocr: blob.ocr,
        }
    }
}
