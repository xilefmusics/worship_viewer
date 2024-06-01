use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerItem {
    Blob(String),
    Chords(chordlib::types::Song),
}

impl Default for PlayerItem {
    fn default() -> Self {
        Self::Blob(String::default())
    }
}
