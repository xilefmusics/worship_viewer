use chordlib::inputs::{chord_pro, ultimate_guitar};
use chordlib::outputs::{FormatChordPro, FormatOutputLines, OutputLine};
use chordlib::types::{SimpleChord, Song as SongData};
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
    pub fn import_ultimate_guitar() -> Result<Self, chordlib::Error> {
        Ok(Self {
            id: None,
            not_a_song: false,
            blobs: vec![],
            data: ultimate_guitar::load_html("")?,
        })
    }

    pub fn format_chord_pro(&self, key: Option<SimpleChord>, language: Option<usize>) -> String {
        (&(self.data)).format_chord_pro(key, language)
    }

    pub fn format_output_lines(
        &self,
        key: Option<SimpleChord>,
        language: Option<usize>,
    ) -> Vec<OutputLine> {
        (&(self.data)).format_output_lines(key, language)
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
