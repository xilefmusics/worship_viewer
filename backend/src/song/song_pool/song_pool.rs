use std::path::PathBuf;

use super::super::{Error, Song, SongIntern};
use super::{SongPoolLocal, SongPoolRemote};
use crate::setlist::SetlistItem;

pub enum SongPool {
    Local(SongPoolLocal),
    Remote(SongPoolRemote),
}

impl SongPool {
    pub fn new_local(path: PathBuf) -> Result<Self, Error> {
        Ok(Self::Local(SongPoolLocal::new(path)?))
    }

    pub fn new_remote(url: String) -> Self {
        Self::Remote(SongPoolRemote::new(url))
    }

    pub fn lazy_load_file(path: PathBuf, key: &str) -> Result<Song, Error> {
        SongIntern::load(path).map(|song| song.to_section_song(key))
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Result<Option<Song>, Error> {
        match self {
            Self::Local(song_pool) => Ok(song_pool.get(setlist_item)),
            Self::Remote(song_pool) => song_pool.get(setlist_item),
        }
    }

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        match self {
            Self::Local(song_pool) => Ok(song_pool.titles()),
            Self::Remote(song_pool) => song_pool.titles(),
        }
    }

    pub fn reload(&self, setlist_item: &SetlistItem) -> Result<(), Error> {
        match self {
            Self::Local(song_pool) => song_pool.reload(setlist_item),
            Self::Remote(_) => Err(Error::Other("remote edit not yet implemented".to_string())),
        }
    }

    pub fn edit(&self, setlist_item: &SetlistItem) -> Result<(), Error> {
        match self {
            Self::Local(song_pool) => song_pool.edit(setlist_item),
            Self::Remote(_) => Err(Error::Other("remote edit not yet implemented".to_string())),
        }
    }

    pub fn transpose(&self, setlist_item: &SetlistItem) -> Result<(), Error> {
        match self {
            Self::Local(song_pool) => song_pool.transpose(setlist_item),
            Self::Remote(_) => Err(Error::Other("remote edit not yet implemented".to_string())),
        }
    }

    pub fn create(&self, song: Song) -> Result<(), Error> {
        match self {
            Self::Local(song_pool) => song_pool.create(song),
            Self::Remote(_) => Err(Error::Other(
                "remote create not yet implemented".to_string(),
            )),
        }
    }
}
