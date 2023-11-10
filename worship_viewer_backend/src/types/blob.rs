use crate::error::AppError;
use crate::types::{string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileType {
    #[serde(rename(deserialize = "image/png", serialize = "image/png"))]
    PNG,
    #[serde(rename(deserialize = "image/jpeg", serialize = "image/jpeg"))]
    JPEG,
}

impl FileType {
    pub fn file_ending(&self) -> &'static str {
        match self {
            Self::PNG => ".png",
            Self::JPEG => ".jpeg",
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

impl IdGetter for BlobDatabase {
    fn get_id_first(&self) -> String {
        self.id.tb.clone()
    }
    fn get_id_second(&self) -> String {
        self.id.id.to_string()
    }
    fn get_id_full(&self) -> String {
        format!("{}:{}", self.get_id_first(), self.get_id_second())
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
            group: format!("{}:{}", self.group.tb, self.group.id.to_string()),
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
