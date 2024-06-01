use super::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlobSong {
    pub title: String,
    pub nr: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
}
