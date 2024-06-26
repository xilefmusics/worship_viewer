use crate::database::{Database, Select};
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileType {
    #[serde(rename(deserialize = "image/png", serialize = "image/png"))]
    PNG,
    #[serde(rename(deserialize = "image/jpeg", serialize = "image/jpeg"))]
    JPEG,
    #[serde(rename(deserialize = "image/svg", serialize = "image/svg"))]
    SVG,
}

impl FileType {
    pub fn file_ending(&self) -> &'static str {
        match self {
            Self::PNG => ".png",
            Self::JPEG => ".jpeg",
            Self::SVG => ".svg",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub id: String,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    pub ocr: String,
    pub group: String,
    pub tags: Vec<String>,
}

impl Blob {
    pub fn file_name(&self) -> Result<String, AppError> {
        Ok(format!("{}{}", self.id, self.file_type.file_ending(),))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlobDatabase {
    pub id: RecordId,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    pub ocr: String,
    pub group: RecordId,
    pub tags: Vec<String>,
}

impl BlobDatabase {
    pub async fn select<'a>(select: Select<'a>) -> Result<Vec<Blob>, AppError> {
        Ok(select
            .table("blob")
            .query::<Self>()
            .await?
            .into_iter()
            .map(|blob| blob.into())
            .collect::<Vec<Blob>>())
    }

    pub async fn create(db: &Database, blobs: Vec<Blob>) -> Result<Vec<Blob>, AppError> {
        Ok(db
            .create_vec(
                "blob",
                blobs
                    .clone()
                    .into_iter()
                    .map(|blob| BlobDatabase::try_from(blob))
                    .collect::<Result<Vec<BlobDatabase>, AppError>>()?,
            )
            .await?
            .into_iter()
            .map(|blob| blob.into())
            .collect::<Vec<Blob>>())
    }
}

impl IdGetter for BlobDatabase {
    fn get_id_second(&self) -> String {
        self.id.id.to_string()
    }
    fn get_id_full(&self) -> String {
        record2string(&self.id)
    }
}

impl Into<Blob> for BlobDatabase {
    fn into(self) -> Blob {
        Blob {
            id: self.get_id_full(),
            file_type: self.file_type,
            width: self.width,
            height: self.height,
            ocr: self.ocr,
            group: record2string(&self.group),
            tags: self.tags,
        }
    }
}

impl TryFrom<Blob> for BlobDatabase {
    type Error = AppError;

    fn try_from(other: Blob) -> Result<Self, Self::Error> {
        Ok(BlobDatabase {
            id: string2record(&other.id)?,
            file_type: other.file_type,
            width: other.width,
            height: other.height,
            ocr: other.ocr,
            group: string2record(&other.group)?,
            tags: other.tags,
        })
    }
}
