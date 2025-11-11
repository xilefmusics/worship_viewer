use super::FileType;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Blob {
    pub id: String,
    pub owner: String,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
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
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateBlob {
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    pub ocr: String,
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
