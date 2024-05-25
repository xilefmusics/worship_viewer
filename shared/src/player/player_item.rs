use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerItem {
    Image(String),
    Pdf(String),
    Chords(String),
}
