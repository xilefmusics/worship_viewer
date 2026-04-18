use super::Song;
use chordlib::types::SimpleChord;
use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(feature = "backend", schema(as = SongLink))]
pub struct Link {
    /// Song record id.
    pub id: String,
    /// Optional display position in the parent list (e.g. `1`, `2a`).
    pub nr: Option<String>,
    /// Musical key for this slot (e.g. `G`, `Am`, `F#m`).
    #[cfg_attr(feature = "backend", schema(value_type = Option<String>))]
    pub key: Option<SimpleChord>,
}

pub struct LinkOwned {
    pub song: Song,
    pub nr: Option<String>,
    pub key: Option<SimpleChord>,
    pub liked: bool,
}
