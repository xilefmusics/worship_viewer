use chordlib::inputs::{chord_pro, ultimate_guitar};
use chordlib::outputs::{FormatChordPro, FormatHTML};
use chordlib::types::{ChordRepresentation, SimpleChord, Song as SongData};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Song {
    pub id: String,
    pub owner: String,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    #[cfg_attr(feature = "backend", schema(value_type = Object, additional_properties = true))]
    pub data: SongData,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct CreateSong {
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    #[cfg_attr(feature = "backend", schema(value_type = Object, additional_properties = true))]
    pub data: SongData,
}

impl TryFrom<&str> for CreateSong {
    type Error = chordlib::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            not_a_song: false,
            blobs: vec![],
            data: chord_pro::load_string(s)?,
        })
    }
}

impl TryFrom<&str> for Song {
    type Error = chordlib::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CreateSong::try_from(s).map(Into::into)
    }
}

impl CreateSong {
    pub fn import_ultimate_guitar(s: &str) -> Result<Self, chordlib::Error> {
        Ok(Self {
            not_a_song: false,
            blobs: vec![],
            data: ultimate_guitar::load_html(s)?,
        })
    }

    pub fn format_chord_pro(
        &self,
        representation: Option<&ChordRepresentation>,
        key: Option<&SimpleChord>,
        language: Option<usize>,
        worship_pro_features: bool,
    ) -> String {
        (&self.data).format_chord_pro(key, representation, language, worship_pro_features)
    }

    pub fn format_html(
        &self,
        key: Option<&SimpleChord>,
        representation: Option<&ChordRepresentation>,
        language: Option<usize>,
        scale: Option<f32>,
    ) -> (String, String) {
        (&self.data).format_html_page(key, representation, language, scale)
    }
}

impl Song {
    pub fn import_ultimate_guitar(s: &str) -> Result<Self, chordlib::Error> {
        CreateSong::import_ultimate_guitar(s).map(Into::into)
    }

    pub fn format_chord_pro(
        &self,
        representation: Option<&ChordRepresentation>,
        key: Option<&SimpleChord>,
        language: Option<usize>,
        worship_pro_features: bool,
    ) -> String {
        (&self.data).format_chord_pro(key, representation, language, worship_pro_features)
    }

    pub fn format_html(
        &self,
        key: Option<&SimpleChord>,
        representation: Option<&ChordRepresentation>,
        language: Option<usize>,
        scale: Option<f32>,
    ) -> (String, String) {
        (&self.data).format_html_page(key, representation, language, scale)
    }
}

impl From<CreateSong> for Song {
    fn from(value: CreateSong) -> Self {
        Self {
            id: String::new(),
            owner: String::new(),
            not_a_song: value.not_a_song,
            blobs: value.blobs,
            data: value.data,
        }
    }
}

impl From<Song> for CreateSong {
    fn from(value: Song) -> Self {
        Self {
            not_a_song: value.not_a_song,
            blobs: value.blobs,
            data: value.data,
        }
    }
}
