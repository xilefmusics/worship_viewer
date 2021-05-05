use std::path::PathBuf;

use super::super::{Error, Song, SongIntern};
use super::{SongPoolDist, SongPoolLocal};
use crate::setlist::SetlistItem;

pub enum SongPool {
    Local(SongPoolLocal),
    Dist(SongPoolDist),
}

impl SongPool {
    pub fn new_local(path: &PathBuf) -> Result<Self, Error> {
        Ok(Self::Local(SongPoolLocal::new(path)?))
    }

    pub fn new_dist(url: String) -> Self {
        Self::Dist(SongPoolDist::new(url))
    }

    pub fn lazy_load_file(path: PathBuf, key: &str) -> Result<Song, Error> {
        SongIntern::load(path).map(|song| song.to_section_song(key))
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Result<Option<Song>, Error> {
        match self {
            Self::Local(song_pool) => Ok(song_pool.get(setlist_item)),
            Self::Dist(song_pool) => song_pool.get(setlist_item),
        }
    }

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        match self {
            Self::Local(song_pool) => Ok(song_pool.titles()),
            Self::Dist(song_pool) => song_pool.titles(),
        }
    }
}
