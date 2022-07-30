use std::path::PathBuf;
use std::sync::Arc;

use crate::song::SongPool;

use super::{Error, Setlist, SetlistPoolLocal, SetlistPoolRemote};

pub enum SetlistPool {
    Local(SetlistPoolLocal),
    Remote(SetlistPoolRemote),
}

impl SetlistPool {
    pub fn new_local(path: PathBuf, song_pool: Arc<SongPool>) -> Result<Self, Error> {
        Ok(Self::Local(SetlistPoolLocal::new(path, song_pool)?))
    }

    pub fn new_remote(url: String) -> Result<Self, Error> {
        Ok(Self::Remote(SetlistPoolRemote::new(url)))
    }

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        match self {
            Self::Local(setlist_pool) => Ok(setlist_pool.titles()),
            Self::Remote(setlist_pool) => setlist_pool.titles(),
        }
    }

    pub fn true_titles(&self) -> Result<Vec<String>, Error> {
        match self {
            Self::Local(setlist_pool) => Ok(setlist_pool.true_titles()),
            Self::Remote(setlist_pool) => setlist_pool.true_titles(),
        }
    }

    pub fn all_songs(&self) -> Result<Setlist, Error> {
        match self {
            Self::Local(setlist_pool) => setlist_pool.all_songs(),
            Self::Remote(setlist_pool) => setlist_pool.all_songs(),
        }
    }

    pub fn get(&self, title: String) -> Result<Option<Setlist>, Error> {
        match self {
            Self::Local(setlist_pool) => setlist_pool.get(title),
            Self::Remote(setlist_pool) => setlist_pool.get(title),
        }
    }

    pub fn get_first(&self) -> Result<Option<Setlist>, Error> {
        match self {
            Self::Local(setlist_pool) => Ok(setlist_pool.get_first()),
            Self::Remote(setlist_pool) => setlist_pool.get_first(),
        }
    }

    pub fn update_setlist(&self, setlist: Setlist) -> Result<(), Error> {
        match self {
            Self::Local(setlist_pool) => setlist_pool.update_setlist(setlist),
            Self::Remote(_) => Err(Error::Other("remote mut not yet implemented".to_string())),
        }
    }

    pub fn delete_setlist(&self, title: String) -> Result<Option<()>, Error> {
        match self {
            Self::Local(setlist_pool) => setlist_pool.delete_setlist(title),
            Self::Remote(_) => Err(Error::Other("remote mut not yet implemented".to_string())),
        }
    }
}
