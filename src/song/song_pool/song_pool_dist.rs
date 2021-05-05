use std::path::PathBuf;

use super::super::{Error, Song};
use crate::setlist::SetlistItem;

pub struct SongPoolDist {
    _songs: Vec<Song>,
}

impl SongPoolDist {
    pub fn new(_path: &PathBuf) -> Result<Self, Error> {
        Err(Error::IO("Todo".to_string()))
    }

    pub fn get(&self, _setlist_item: &SetlistItem) -> Option<Song> {
        None
    }

    pub fn titles(&self) -> Vec<String> {
        vec![]
    }
}
