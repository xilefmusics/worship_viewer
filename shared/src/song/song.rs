use chordlib::inputs::chord_pro;
use chordlib::outputs::{FormatChordPro, FormatHTML};
use chordlib::types::{ChordRepresentation, SimpleChord, Song as SongData};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct SongUserSpecificAddons {
    pub liked: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Song {
    pub id: String,
    pub owner: String,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    #[cfg_attr(feature = "backend", schema(value_type = Object, additional_properties = true))]
    pub data: SongData,
    pub user_specific_addons: SongUserSpecificAddons,
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
            user_specific_addons: SongUserSpecificAddons::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_song_json_uses_metadata_vectors() {
        let json = r#"{"not_a_song":false,"blobs":[],"data":{"titles":["Hello"],"artists":["A"],"languages":["en"],"sections":[]}}"#;
        let s: CreateSong = serde_json::from_str(json).unwrap();
        assert_eq!(s.data.title(), "Hello");
        assert_eq!(s.data.artist(), "A");
        assert_eq!(s.data.language(), "en");
    }
}
