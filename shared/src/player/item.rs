use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Item {
    Image(String),
    Pdf(String),
    Chords(String),
}
