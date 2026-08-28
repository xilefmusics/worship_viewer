use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use shared::{MediaAsset, MediaAssetKind, MediaAssetStatus};

use crate::database::record_id_string;

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct MediaAssetRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<RecordId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_id: Option<RecordId>,
    pub kind: String,
    pub content_type: String,
    pub byte_length: u64,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<surrealdb::types::Datetime>,
}

#[derive(Clone, Debug)]
pub struct CreateFinalAsset {
    pub owner: surrealdb::types::RecordId,
    pub media_id: surrealdb::types::RecordId,
    pub kind: MediaAssetKind,
    pub content_type: String,
    pub byte_length: u64,
    pub etag: String,
}

#[derive(Clone, Debug)]
pub struct CreateStagingAsset {
    pub owner: surrealdb::types::RecordId,
    pub media_id: surrealdb::types::RecordId,
    pub kind: MediaAssetKind,
    pub content_type: String,
    pub byte_length: u64,
    pub operation_id: String,
    pub etag: String,
}

impl MediaAssetRecord {
    pub fn into_asset(self) -> Result<MediaAsset, crate::error::AppError> {
        let kind = MediaAssetKind::parse(&self.kind)
            .ok_or_else(|| crate::error::AppError::database("invalid media asset kind"))?;
        let status = match self.status.as_str() {
            "staging" => MediaAssetStatus::Staging,
            "final" => MediaAssetStatus::Final,
            _ => {
                return Err(crate::error::AppError::database(
                    "invalid media asset status",
                ));
            }
        };
        Ok(MediaAsset {
            id: self.id.map(|r| record_id_string(&r)).unwrap_or_default(),
            owner: self.owner.map(|r| record_id_string(&r)).unwrap_or_default(),
            media_id: self
                .media_id
                .map(|r| record_id_string(&r))
                .unwrap_or_default(),
            kind,
            content_type: self.content_type,
            byte_length: self.byte_length,
            status,
            operation_id: self.operation_id,
            etag: self.etag,
        })
    }

    pub fn from_staging(
        owner: RecordId,
        media_id: RecordId,
        kind: MediaAssetKind,
        content_type: String,
        byte_length: u64,
        operation_id: String,
        etag: String,
    ) -> Self {
        Self {
            id: None,
            owner: Some(owner),
            media_id: Some(media_id),
            kind: kind.as_str().into(),
            content_type,
            byte_length,
            status: "staging".into(),
            operation_id: Some(operation_id),
            etag: Some(etag),
            created_at: Some(Utc::now().into()),
        }
    }

    pub fn from_final(
        owner: RecordId,
        media_id: RecordId,
        kind: MediaAssetKind,
        content_type: String,
        byte_length: u64,
        etag: String,
    ) -> Self {
        Self {
            id: None,
            owner: Some(owner),
            media_id: Some(media_id),
            kind: kind.as_str().into(),
            content_type,
            byte_length,
            status: "final".into(),
            operation_id: None,
            etag: Some(etag),
            created_at: Some(Utc::now().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_into_asset() {
        let record = MediaAssetRecord {
            id: Some(RecordId::new("media_asset", "a1")),
            owner: Some(RecordId::new("team", "t1")),
            media_id: Some(RecordId::new("media", "m1")),
            kind: "video".into(),
            content_type: "video/mp4".into(),
            byte_length: 42,
            status: "final".into(),
            operation_id: None,
            etag: Some("W/\"abc\"".into()),
            created_at: None,
        };
        let asset = record.into_asset().unwrap();
        assert_eq!(asset.id, "a1");
        assert_eq!(asset.status, MediaAssetStatus::Final);
    }
}
