use std::path::PathBuf;

use super::{Error, Song, SongPoolLocal};
use crate::setlist::SetlistItem;

pub enum SongPool {
    Local(SongPoolLocal),
}

impl SongPool {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let song_pool_local = SongPoolLocal::new(path)?;
        Ok(Self::Local(song_pool_local))
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Option<Song> {
        match self {
            Self::Local(song_pool_local) => song_pool_local.get(setlist_item),
        }
    }

    pub fn titles(&self) -> Vec<String> {
        match self {
            Self::Local(song_pool_local) => song_pool_local.titles(),
        }
    }
}
