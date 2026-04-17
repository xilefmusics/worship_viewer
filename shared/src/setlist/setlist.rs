use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Setlist {
    pub id: String,
    pub owner: String,
    pub title: String,
    pub songs: Vec<SongLink>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateSetlist {
    pub title: String,
    pub songs: Vec<SongLink>,
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
