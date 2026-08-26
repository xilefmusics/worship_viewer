use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

/// Processing state for a media resource or a pending content revision.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum MediaStatus {
    Processing,
    Ready,
    Failed,
}

/// Stable, sanitized processing failure exposed to clients.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct MediaProcessingError {
    pub code: String,
    pub detail: String,
}

/// A content replacement that has not yet replaced active Ready content.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct MediaPendingRevision {
    pub operation: String,
    pub status: MediaStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_error: Option<MediaProcessingError>,
    /// Draft slide-deck pages. Empty for audio/video revisions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<MediaStagedDeckPage>,
}

impl MediaPendingRevision {
    pub fn processing(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            status: MediaStatus::Processing,
            processing_error: None,
            pages: Vec::new(),
        }
    }

    pub fn failed(operation: impl Into<String>, processing_error: MediaProcessingError) -> Self {
        Self {
            operation: operation.into(),
            status: MediaStatus::Failed,
            processing_error: Some(processing_error),
            pages: Vec::new(),
        }
    }
}

/// A page in a staged (not yet committed) slide-deck revision.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct MediaStagedDeckPage {
    pub id: String,
    pub blob_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum LivestreamType {
    Hls,
    Direct,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum MediaContent {
    SlideDeck {
        pages: Vec<MediaDeckPage>,
    },
    Video {
        blob_id: String,
        duration_ms: u64,
        width: u32,
        height: u32,
    },
    Audio {
        blob_id: String,
        duration_ms: u64,
    },
    #[serde(rename = "youtube")]
    YouTube {
        video_id: String,
        canonical_url: String,
    },
    Livestream {
        url: String,
        stream_type: LivestreamType,
    },
    WebPage {
        url: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct MediaDeckPage {
    pub blob_id: String,
}

/// Upload kind declared at create time before active content exists.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum DeclaredMediaKind {
    SlideDeck,
    Video,
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Media {
    pub id: String,
    pub owner: String,
    pub title: String,
    pub status: MediaStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<MediaContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_revision: Option<MediaPendingRevision>,
    /// Present for uploaded/deck shells until Ready content is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_kind: Option<DeclaredMediaKind>,
}

/// Create a synchronously validated URL-backed media resource.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateMedia {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub title: String,
    pub content: CreateMediaContent,
}

/// URL content accepted by E5.1. Uploaded/deck tags are reserved on
/// [`MediaContent`] but cannot be fabricated through this request.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum CreateMediaContent {
    SlideDeck,
    Video,
    Audio,
    #[serde(rename = "youtube")]
    YouTube {
        url: String,
    },
    Livestream {
        url: String,
    },
    WebPage {
        url: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct UpdateMedia {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<CreateMediaContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct DuplicateMedia {
    /// Destination team. Omit to keep the source owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Title for the copy. Omit to reuse the source title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Finalize a staged slide-deck revision in the operator-selected order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CommitDeck {
    pub operation: String,
    pub page_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_content_tags_round_trip() {
        let values = [
            serde_json::json!({"type":"slide_deck","pages":[{"blob_id":"b1"}]}),
            serde_json::json!({"type":"video","blob_id":"b1","duration_ms":1,"width":2,"height":3}),
            serde_json::json!({"type":"audio","blob_id":"b1","duration_ms":1}),
            serde_json::json!({"type":"youtube","video_id":"dQw4w9WgXcQ","canonical_url":"https://www.youtube.com/watch?v=dQw4w9WgXcQ"}),
            serde_json::json!({"type":"livestream","url":"https://example.com/live.m3u8","stream_type":"hls"}),
            serde_json::json!({"type":"web_page","url":"https://example.com/"}),
        ];
        for value in values {
            let parsed: MediaContent = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), value);
        }
    }

    #[test]
    fn lifecycle_optional_fields_and_unknown_tags_are_stable() {
        let value = serde_json::json!({
            "id":"m1", "owner":"t1", "title":"Video", "status":"ready",
            "content":{"type":"youtube","video_id":"dQw4w9WgXcQ","canonical_url":"https://www.youtube.com/watch?v=dQw4w9WgXcQ"}
        });
        let media: Media = serde_json::from_value(value).unwrap();
        assert!(media.pending_revision.is_none());
        assert!(
            serde_json::from_value::<MediaContent>(serde_json::json!({"type":"future"})).is_err()
        );
    }

    #[test]
    fn create_upload_kinds_and_declared_kind_round_trip() {
        for (kind, tag, expected) in [
            ("video", "video", DeclaredMediaKind::Video),
            ("audio", "audio", DeclaredMediaKind::Audio),
            ("slide_deck", "slide_deck", DeclaredMediaKind::SlideDeck),
        ] {
            let create: CreateMediaContent =
                serde_json::from_value(serde_json::json!({"type": tag})).unwrap();
            assert_eq!(
                serde_json::to_value(create).unwrap(),
                serde_json::json!({"type": tag})
            );
            let media: Media = serde_json::from_value(serde_json::json!({
                "id":"m1","owner":"t1","title":"Clip","status":"processing",
                "declared_kind": kind
            }))
            .unwrap();
            assert_eq!(media.declared_kind, Some(expected));
        }
    }

    #[test]
    fn pending_revision_pages_default_and_commit_round_trip() {
        let pending: MediaPendingRevision = serde_json::from_value(serde_json::json!({
            "operation": "rev1",
            "status": "ready",
            "pages": [{"id": "p1", "blob_id": "b1"}]
        }))
        .unwrap();
        assert_eq!(pending.pages[0].id, "p1");
        let commit: CommitDeck = serde_json::from_value(serde_json::json!({
            "operation": "rev1",
            "page_ids": ["p1"]
        }))
        .unwrap();
        assert_eq!(commit.page_ids, vec!["p1"]);
        let av: MediaPendingRevision = serde_json::from_value(serde_json::json!({
            "operation": "op1",
            "status": "processing"
        }))
        .unwrap();
        assert!(av.pages.is_empty());
    }
}
