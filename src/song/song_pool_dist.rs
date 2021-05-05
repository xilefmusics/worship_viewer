use std::path::PathBuf;

use super::{Error, Song};

pub struct SongPoolDist {
    _songs: Vec<Song>,
}

impl SongPoolDist {
    pub fn new(_path: &PathBuf) -> Result<Self, Error> {
        Err(Error::IO("Todo".to_string()))
    }

    pub fn get(&self, _title: String) -> Option<Song> {
        None
    }

    pub fn titles(&self) -> Vec<String> {
        vec![]
    }
}
