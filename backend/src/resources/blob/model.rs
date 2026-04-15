use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use shared::blob::{Blob, CreateBlob};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Thing>,
    pub file_type: shared::blob::FileType,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub ocr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Datetime>,
}

impl BlobRecord {
    pub fn into_blob(self) -> Blob {
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

    pub fn from_payload(
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::blob::FileType;

    #[test]
    fn blob_record_from_payload_into_blob() {
        let id = Thing::from(("blob".to_owned(), "b99".to_owned()));
        let owner = Thing::from(("team".to_owned(), "tm".to_owned()));
        let record = BlobRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            None,
            CreateBlob {
                file_type: FileType::SVG,
                width: 640,
                height: 480,
                ocr: "text".into(),
            },
        );
        let b = record.into_blob();
        assert_eq!(b.id, "b99");
        assert_eq!(b.owner, "tm");
        assert_eq!(b.file_type, FileType::SVG);
        assert_eq!(b.width, 640);
        assert_eq!(b.height, 480);
        assert_eq!(b.ocr, "text");
    }
}
