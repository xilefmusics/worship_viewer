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

    pub fn from_str(s: &str) -> Self {
        match s {
            "Ab" => Self::Ab,
            "A" => Self::A,
            "A#" => Self::As,
            "Bb" => Self::Bb,
            "B" => Self::B,
            "B#" => Self::Bs,
            "Cb" => Self::Cb,
            "C" => Self::C,
            "C#" => Self::Cs,
            "Db" => Self::Db,
            "D" => Self::D,
            "D#" => Self::Ds,
            "Eb" => Self::Eb,
            "E" => Self::E,
            "E#" => Self::Es,
            "Fb" => Self::Fb,
            "F" => Self::F,
            "F#" => Self::Fs,
            "Gb" => Self::Gb,
            "G" => Self::G,
            "G#" => Self::Gs,
            _ => Self::NotAKey,
        }
    }
}
