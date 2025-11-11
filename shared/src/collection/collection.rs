use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Collection {
    pub id: Option<String>,
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
}
