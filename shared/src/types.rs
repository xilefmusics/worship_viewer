use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Key {
    Ab,
    A,
    #[serde(rename(deserialize = "A#", serialize = "A#"))]
    As,
    Bb,
    B,
    #[serde(rename(deserialize = "B#", serialize = "B#"))]
    Bs,
    Cb,
    C,
    #[serde(rename(deserialize = "C#", serialize = "C#"))]
    Cs,
    Db,
    D,
    #[serde(rename(deserialize = "D#", serialize = "D#"))]
    Ds,
    Eb,
    E,
    #[serde(rename(deserialize = "E#", serialize = "E#"))]
    Es,
    Fb,
    F,
    #[serde(rename(deserialize = "F#", serialize = "F#"))]
    Fs,
    Gb,
    G,
    #[serde(rename(deserialize = "G#", serialize = "G#"))]
    Gs,
    #[serde(rename(deserialize = "", serialize = ""))]
    NotAKey,
}

impl Key {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Ab => "Ab",
            Self::A => "A",
            Self::As => "A#",
            Self::Bb => "Bb",
            Self::B => "B",
            Self::Bs => "B#",
            Self::Cb => "Cb",
            Self::C => "C",
            Self::Cs => "C#",
            Self::Db => "Db",
            Self::D => "D",
            Self::Ds => "D#",
            Self::Eb => "Eb",
            Self::E => "E",
            Self::Es => "E#",
            Self::Fb => "Fb",
            Self::F => "F",
            Self::Fs => "F#",
            Self::Gb => "Gb",
            Self::G => "G",
            Self::Gs => "G#",
            Self::NotAKey => "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub nr: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    pub collection: String,
    pub group: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub songs: Vec<String>,
    pub cover: String,
    pub group: String,
    pub tags: Vec<String>,
}

