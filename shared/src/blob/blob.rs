use super::FileType;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

/// Cross-resource reference to a blob (opaque `id` as returned by the API).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct BlobLink {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Blob {
    pub id: String,
    pub owner: String,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    /// OCR or extracted text used for search (may be empty).
    pub ocr: String,
}

impl Blob {
    pub fn file_name(&self) -> Option<String> {
        if self.id.is_empty() {
            None
        } else {
            Some(format!("{}{}", self.id, self.file_type.file_ending()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateBlob {
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    /// OCR or extracted text used for search (may be empty).
    pub ocr: String,
}

/// Full replacement body for `PUT /api/v1/blobs/{id}` metadata (same shape as [`CreateBlob`]; does not upload bytes).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct UpdateBlob {
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    /// OCR or extracted text used for search (may be empty).
    pub ocr: String,
}

impl From<UpdateBlob> for CreateBlob {
    fn from(value: UpdateBlob) -> Self {
        Self {
            file_type: value.file_type,
            width: value.width,
            height: value.height,
            ocr: value.ocr,
        }
    }
}

/// Partial update for a blob. Absent fields are left unchanged.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct PatchBlob {
    pub file_type: Option<FileType>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// OCR or extracted text used for search (may be empty).
    pub ocr: Option<String>,
}

impl From<Blob> for CreateBlob {
    fn from(value: Blob) -> Self {
        Self {
            file_type: value.file_type,
            width: value.width,
            height: value.height,
            ocr: value.ocr,
        }
    }
}
