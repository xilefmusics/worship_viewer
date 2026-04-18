//! OpenAPI schema for the ChordPro-derived song payload (mirrors [`chordlib::types::Song`] wire JSON).
//! Runtime types use `chordlib::types::Song` directly; this type exists for `utoipa` only.

use std::collections::BTreeMap;

#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;
#[cfg(feature = "backend")]
use utoipa::ToSchema;

/// ChordPro-derived metadata and content (titles, tags, [`sections`](https://chordpro.org/) as structured blocks).
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(
        example = json!({
            "titles": ["Amazing Grace"],
            "subtitle": null,
            "copyright": null,
            "key": null,
            "artists": [],
            "languages": ["en"],
            "tempo": null,
            "time": null,
            "tags": {},
            "sections": []
        })
    )
)]
pub struct SongDataSchema {
    /// Primary and alternate titles from ChordPro `{title}` / `{title:N}` directives.
    #[cfg_attr(feature = "backend", schema(example = json!(["Example Hymn"])))]
    pub titles: Vec<String>,
    pub subtitle: Option<String>,
    pub copyright: Option<String>,
    /// Musical key; chord symbols use ChordPro conventions.
    #[cfg_attr(feature = "backend", schema(value_type = Option<String>, example = json!("G")))]
    pub key: Option<chordlib::types::SimpleChord>,
    pub artists: Vec<String>,
    /// BCP 47 language tags (e.g. `en`, `de-CH`).
    #[cfg_attr(feature = "backend", schema(example = json!(["en"])))]
    pub languages: Vec<String>,
    /// Tempo in BPM (beats per minute).
    pub tempo: Option<u32>,
    /// Time signature as `(numerator, denominator)` (e.g. 4/4).
    #[cfg_attr(feature = "backend", schema(value_type = Option<[u32; 2]>, example = json!([4, 4])))]
    pub time: Option<(u32, u32)>,
    /// Custom meta tags from ChordPro `{meta: name value}` pairs.
    #[cfg_attr(feature = "backend", schema(additional_properties = true))]
    pub tags: BTreeMap<String, String>,
    /// Structured sections (verse, chorus, etc.) with lyric lines and chords.
    #[cfg_attr(feature = "backend", schema(value_type = Vec<Object>))]
    pub sections: Vec<chordlib::types::Section>,
}
