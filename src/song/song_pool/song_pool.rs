use std::path::PathBuf;

use super::super::{Error, Song, SongIntern};
use super::{SongPoolDist, SongPoolLocal};
use crate::setlist::SetlistItem;

pub enum SongPool {
    Local(SongPoolLocal),
    Dist(SongPoolDist),
}

impl SongPool {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let song_pool_local = SongPoolLocal::new(path)?;
        Ok(Self::Local(song_pool_local))
    }

    pub fn lazy_load_file(path: PathBuf, key: &str) -> Result<Song, Error> {
        SongIntern::load(path).map(|song| song.to_section_song(key))
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Option<Song> {
        match self {
            Self::Local(song_pool) => song_pool.get(setlist_item),
            Self::Dist(song_pool) => song_pool.get(setlist_item),
        }
    }

    pub fn titles(&self) -> Vec<String> {
        match self {
            Self::Local(song_pool) => song_pool.titles(),
            Self::Dist(song_pool) => song_pool.titles(),
        }
    }
}
