use super::{BlobSong, Key};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: String,
    pub data: SongData,
    pub collection: String,
    pub group: String,
    pub tags: Vec<String>,
}

impl Song {
    pub fn title(&self) -> &str {
        match &self.data {
            SongData::Blob(data) => &data.title,
            SongData::Chord(data) => &data.title,
        }
    }

    pub fn not_a_song(&self) -> bool {
        match &self.data {
            SongData::Blob(data) => data.not_a_song,
            SongData::Chord(_) => false,
        }
    }

    pub fn key(&self) -> Key {
        match &self.data {
            SongData::Blob(data) => data.key.clone(),
            SongData::Chord(data) => {
                Key::from_str(data.key.format(&chordlib::types::SimpleChord::default()))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SongData {
    Blob(BlobSong),
    Chord(chordlib::types::Song),
}
