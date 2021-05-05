use serde::Serialize;

use super::line::Section;

#[derive(Debug, Clone, Serialize)]
pub struct Song {
    pub title: String,
    pub key: String,
    pub sections: Vec<Section>,
}
