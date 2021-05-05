use serde::{Deserialize, Serialize};

use super::line::Section;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub key: String,
    pub sections: Vec<Section>,
}
