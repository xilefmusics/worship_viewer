use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, Kind, RecordId, SurrealValue, Value, kind};

use shared::blob::{Blob, CreateBlob, FileType};

use crate::database::record_id_string;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileTypeField(pub FileType);

impl SurrealValue for FileTypeField {
    fn kind_of() -> Kind {
        kind!(any)
    }

    fn is_value(_value: &Value) -> bool {
        true
    }

    fn into_value(self) -> Value {
        let j = serde_json::to_value(self.0).unwrap_or(serde_json::Value::Null);
        j.into_value()
    }

    fn from_value(value: Value) -> surrealdb::Result<Self> {
        let j = serde_json::Value::from_value(value)?;
        serde_json::from_value(j)
            .map(FileTypeField)
            .map_err(|e| surrealdb::Error::internal(e.to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct BlobRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<RecordId>,
    pub file_type: FileTypeField,
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
            id: self.id.map(|r| record_id_string(&r)).unwrap_or_default(),
            owner: self.owner.map(|r| record_id_string(&r)).unwrap_or_default(),
            file_type: self.file_type.0,
            width: self.width,
            height: self.height,
            ocr: self.ocr,
        }
    }

    pub fn from_payload(
        id: Option<RecordId>,
        owner: Option<RecordId>,
        created_at: Option<Datetime>,
        blob: CreateBlob,
    ) -> Self {
        let CreateBlob {
            file_type,
            width,
            height,
            ocr,
            ..
        } = blob;
        Self {
            id,
            owner,
            file_type: FileTypeField(file_type),
            width,
            height,
            ocr,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_record_from_payload_into_blob() {
        let id = RecordId::new("blob", "b99");
        let owner = RecordId::new("team", "tm");
        let record = BlobRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            None,
            CreateBlob {
                owner: None,
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
