use chordlib::inputs::chord_pro;
use chordlib::outputs::{FormatChordPro, FormatHTML};
use chordlib::types::{ChordRepresentation, SimpleChord, Song as SongData};
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;

#[cfg(feature = "backend")]
use utoipa::ToSchema;

fn deserialize_song_data_compat<'de, D>(deserializer: D) -> Result<SongData, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let mut v = serde_json::Value::deserialize(deserializer)?;
    if let Some(obj) = v.as_object_mut() {
        let titles_empty = match obj.get("titles") {
            None => true,
            Some(t) => t.as_array().map_or(true, Vec::is_empty),
        };
        if titles_empty {
            if let Some(serde_json::Value::String(s)) = obj.remove("title") {
                obj.insert("titles".to_string(), serde_json::json!([s]));
            }
        } else {
            obj.remove("title");
        }

        let artists_empty = match obj.get("artists") {
            None => true,
            Some(t) => t.as_array().map_or(true, Vec::is_empty),
        };
        if artists_empty {
            if let Some(serde_json::Value::String(s)) = obj.remove("artist") {
                if !s.is_empty() {
                    obj.insert("artists".to_string(), serde_json::json!([s]));
                }
            }
        } else {
            obj.remove("artist");
        }

        let languages_empty = match obj.get("languages") {
            None => true,
            Some(t) => t.as_array().map_or(true, Vec::is_empty),
        };
        if languages_empty {
            if let Some(serde_json::Value::String(s)) = obj.remove("language") {
                if !s.is_empty() {
                    obj.insert("languages".to_string(), serde_json::json!([s]));
                }
            }
        } else {
            obj.remove("language");
        }
    }

    SongData::deserialize(v).map_err(D::Error::custom)
}

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
    #[serde(deserialize_with = "deserialize_song_data_compat")]
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
mod compat_tests {
    use super::*;

    #[test]
    fn deserialize_legacy_title_key_into_titles() {
        let json = r#"{"not_a_song":false,"blobs":[],"data":{"title":"Hello","sections":[]}}"#;
        let s: CreateSong = serde_json::from_str(json).unwrap();
        assert_eq!(s.data.title(), "Hello");
        assert_eq!(s.data.titles, vec!["Hello".to_string()]);
    }

    #[test]
    fn deserialize_legacy_artist_and_language_keys() {
        let json = r#"{"not_a_song":false,"blobs":[],"data":{"title":"T","artist":"A","language":"en","sections":[]}}"#;
        let s: CreateSong = serde_json::from_str(json).unwrap();
        assert_eq!(s.data.artists, vec!["A".to_string()]);
        assert_eq!(s.data.languages, vec!["en".to_string()]);
    }

    #[test]
    fn deserialize_prefers_nonempty_titles_over_legacy_title() {
        let json = r#"{"not_a_song":false,"blobs":[],"data":{"title":"Legacy","titles":["A","B"],"sections":[]}}"#;
        let s: CreateSong = serde_json::from_str(json).unwrap();
        assert_eq!(s.data.titles, vec!["A".to_string(), "B".to_string()]);
        assert_eq!(s.data.title(), "A");
    }
}
