use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

/// Source kind for a media-owned asset; selects upload byte limits and processing paths.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum MediaAssetKind {
    Video,
    Audio,
    Image,
    Pdf,
    Svg,
}

impl MediaAssetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Pdf => "pdf",
            Self::Svg => "svg",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "video" => Some(Self::Video),
            "audio" => Some(Self::Audio),
            "image" => Some(Self::Image),
            "pdf" => Some(Self::Pdf),
            "svg" => Some(Self::Svg),
            _ => None,
        }
    }
}

/// Lifecycle of bytes on disk for a media-owned asset.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum MediaAssetStatus {
    /// Upload staging; not readable via delivery endpoints.
    Staging,
    /// Promoted private storage; readable via authenticated delivery.
    Final,
}

/// Metadata for a media-owned asset (no filesystem paths).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct MediaAsset {
    pub id: String,
    pub owner: String,
    pub media_id: String,
    pub kind: MediaAssetKind,
    pub content_type: String,
    pub byte_length: u64,
    pub status: MediaAssetStatus,
    /// Present while `status` is `staging`; cleared after promotion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// Weak ETag for delivery (`W/"..."`); set when promotion completes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_asset_kind_round_trip() {
        for kind in [
            MediaAssetKind::Video,
            MediaAssetKind::Audio,
            MediaAssetKind::Image,
            MediaAssetKind::Pdf,
            MediaAssetKind::Svg,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: MediaAssetKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
            assert_eq!(MediaAssetKind::parse(kind.as_str()), Some(kind));
        }
    }
}
