use crate::song::Song;
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum PlayerItem {
    Blob(String),
    Chords(Song, Option<crate::song::SimpleChord>),
}

impl Default for PlayerItem {
    fn default() -> Self {
        Self::Blob(String::default())
    }
}
