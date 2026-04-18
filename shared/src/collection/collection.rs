use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;
#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({
        "id": "col_example",
        "owner": "usr_example",
        "title": "Sunday worship",
        "cover": "",
        "songs": [{ "id": "song_example", "nr": null, "key": null }]
    }))
)]
pub struct Collection {
    pub id: String,
    pub owner: String,
    pub title: String,
    /// Cover art reference (client-resolved blob id or URL).
    pub cover: String,
    pub songs: Vec<SongLink>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({
        "title": "Sunday worship",
        "cover": "",
        "songs": [{ "id": "song_example", "nr": null, "key": null }]
    }))
)]
pub struct CreateCollection {
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
}

/// Full replacement body for `PUT /api/v1/collections/{id}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct UpdateCollection {
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
}

impl From<UpdateCollection> for CreateCollection {
    fn from(value: UpdateCollection) -> Self {
        Self {
            title: value.title,
            cover: value.cover,
            songs: value.songs,
        }
    }
}

/// Partial update for a collection. Absent fields are left unchanged.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct PatchCollection {
    pub title: Option<String>,
    pub cover: Option<String>,
    pub songs: Option<Vec<SongLink>>,
}

impl From<Collection> for CreateCollection {
    fn from(value: Collection) -> Self {
        Self {
            title: value.title,
            cover: value.cover,
            songs: value.songs,
        }
    }
}
