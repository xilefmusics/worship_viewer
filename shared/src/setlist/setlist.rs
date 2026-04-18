use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;
#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({
        "id": "set_example",
        "owner": "usr_example",
        "title": "Easter Sunday",
        "songs": [{ "id": "song_example", "nr": "1", "key": null }]
    }))
)]
pub struct Setlist {
    pub id: String,
    pub owner: String,
    pub title: String,
    pub songs: Vec<SongLink>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({
        "title": "Easter Sunday",
        "songs": [{ "id": "song_example", "nr": "1", "key": null }]
    }))
)]
pub struct CreateSetlist {
    pub title: String,
    pub songs: Vec<SongLink>,
}

/// Full replacement body for `PUT /api/v1/setlists/{id}`.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct UpdateSetlist {
    pub title: String,
    pub songs: Vec<SongLink>,
}

impl From<CreateSetlist> for UpdateSetlist {
    fn from(value: CreateSetlist) -> Self {
        Self {
            title: value.title,
            songs: value.songs,
        }
    }
}

impl From<UpdateSetlist> for CreateSetlist {
    fn from(value: UpdateSetlist) -> Self {
        Self {
            title: value.title,
            songs: value.songs,
        }
    }
}

/// Partial update for a setlist. Absent fields are left unchanged.
#[derive(Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct PatchSetlist {
    pub title: Option<String>,
    pub songs: Option<Vec<SongLink>>,
}

impl From<Setlist> for CreateSetlist {
    fn from(value: Setlist) -> Self {
        Self {
            title: value.title,
            songs: value.songs,
        }
    }
}
