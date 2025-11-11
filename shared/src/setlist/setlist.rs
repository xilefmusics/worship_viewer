use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Setlist {
    pub id: Option<String>,
    pub title: String,
    pub songs: Vec<SongLink>,
}
