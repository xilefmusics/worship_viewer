use serde::Serialize;

use super::super::line::Section;

#[derive(Debug, Clone, Serialize)]
pub struct SectionSong {
    title: String,
    sections: Vec<Section>,
}

impl SectionSong {
    pub fn new(title: String, sections: Vec<Section>) -> Self {
        Self { title, sections }
    }
}
