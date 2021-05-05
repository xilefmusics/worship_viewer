use serde::Serialize;

use crate::line::Section;

#[derive(Debug, Clone, Serialize)]
pub struct SectionSong {
    pub title: String,
    pub key: String,
    pub sections: Vec<Section>,
}
