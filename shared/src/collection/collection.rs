use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Collection {
    pub id: String,
    pub owner: String,
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateCollection {
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
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
