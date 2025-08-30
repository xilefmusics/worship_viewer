use chordlib::types::SimpleChord;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Link {
    pub id: String,
    pub nr: Option<String>,
    pub key: Option<SimpleChord>,
}
