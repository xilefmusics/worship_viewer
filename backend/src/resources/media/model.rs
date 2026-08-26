use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use shared::media::{DeclaredMediaKind, Media, MediaContent, MediaPendingRevision, MediaStatus};

use crate::database::record_id_string;
use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct MediaWrite {
    pub title: String,
    pub status: MediaStatus,
    pub content: Option<MediaContent>,
    pub pending_revision: Option<MediaPendingRevision>,
    pub declared_kind: Option<DeclaredMediaKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct MediaRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<RecordId>,
    pub title: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_revision_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_kind: Option<String>,
}

impl MediaRecord {
    pub fn from_write(
        id: Option<RecordId>,
        owner: Option<RecordId>,
        value: MediaWrite,
    ) -> Result<Self, AppError> {
        Ok(Self {
            id,
            owner,
            title: value.title,
            status: status_string(value.status).into(),
            content_json: value
                .content
                .map(|v| serde_json::to_string(&v))
                .transpose()
                .map_err(|e| AppError::internal_from_err("media.model", e))?,
            pending_revision_json: value
                .pending_revision
                .map(|v| serde_json::to_string(&v))
                .transpose()
                .map_err(|e| AppError::internal_from_err("media.model", e))?,
            declared_kind: value.declared_kind.map(declared_kind_string),
        })
    }

    pub fn into_media(self) -> Result<Media, AppError> {
        Ok(Media {
            id: self.id.map(|v| record_id_string(&v)).unwrap_or_default(),
            owner: self.owner.map(|v| record_id_string(&v)).unwrap_or_default(),
            title: self.title,
            status: parse_status(&self.status)?,
            content: self
                .content_json
                .map(|v| serde_json::from_str(&v))
                .transpose()
                .map_err(|e| AppError::internal_from_err("media.model", e))?,
            pending_revision: self
                .pending_revision_json
                .map(|v| serde_json::from_str(&v))
                .transpose()
                .map_err(|e| AppError::internal_from_err("media.model", e))?,
            declared_kind: self
                .declared_kind
                .map(|v| parse_declared_kind(&v))
                .transpose()?,
        })
    }
}

fn status_string(status: MediaStatus) -> &'static str {
    match status {
        MediaStatus::Processing => "processing",
        MediaStatus::Ready => "ready",
        MediaStatus::Failed => "failed",
    }
}

fn parse_status(value: &str) -> Result<MediaStatus, AppError> {
    match value {
        "processing" => Ok(MediaStatus::Processing),
        "ready" => Ok(MediaStatus::Ready),
        "failed" => Ok(MediaStatus::Failed),
        _ => Err(AppError::Internal("invalid persisted media status".into())),
    }
}

fn declared_kind_string(kind: DeclaredMediaKind) -> String {
    match kind {
        DeclaredMediaKind::SlideDeck => "slide_deck".into(),
        DeclaredMediaKind::Video => "video".into(),
        DeclaredMediaKind::Audio => "audio".into(),
    }
}

fn parse_declared_kind(value: &str) -> Result<DeclaredMediaKind, AppError> {
    match value {
        "slide_deck" => Ok(DeclaredMediaKind::SlideDeck),
        "video" => Ok(DeclaredMediaKind::Video),
        "audio" => Ok(DeclaredMediaKind::Audio),
        _ => Err(AppError::Internal("invalid persisted declared_kind".into())),
    }
}
