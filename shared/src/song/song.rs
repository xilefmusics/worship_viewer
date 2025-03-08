use chordlib::inputs::{chord_pro, ultimate_guitar};
use chordlib::outputs::{FormatChordPro, FormatHTML};
use chordlib::types::{Key, Song as SongData};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Song {
    pub id: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    pub data: SongData,
}

impl TryFrom<&str> for Song {
    type Error = chordlib::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            id: None,
            not_a_song: false,
            blobs: vec![],
            data: chord_pro::load_string(s)?,
        })
    }
}

impl Song {
    pub fn import_ultimate_guitar(s: &str) -> Result<Self, chordlib::Error> {
        Ok(Self {
            id: None,
            not_a_song: false,
            blobs: vec![],
            data: ultimate_guitar::load_html(s)?,
        })
    }

    pub fn format_worship_pro(&self, key: Option<&Key>, language: Option<usize>) -> String {
        (&(self.data)).format_chord_pro(key, language, true)
    }

    pub fn format_html(
        &self,
        key: Option<&Key>,
        language: Option<usize>,
        scale: Option<f32>,
    ) -> (String, String) {
        (&(self.data)).format_html_page(key, language, scale)
    }
}

#[cfg(feature = "backend")]
impl fancy_surreal::Databasable for Song {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
