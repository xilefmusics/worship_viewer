use std::fs;
use std::path::PathBuf;

use super::{Error, Song, SongIntern};
use crate::setlist::SetlistItem;

pub struct SongPoolLocal {
    songs: Vec<SongIntern>,
}

impl SongPoolLocal {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut songs = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| path.is_ok() && !path.as_ref().unwrap().is_dir())
            .map(|path| SongIntern::load(path?.clone()))
            .collect::<Result<Vec<SongIntern>, Error>>()?;
        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(Self { songs })
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Option<Song> {
        let song = self
            .songs
            .iter()
            .find(|song| song.title == setlist_item.title)?;
        Some(song.to_section_song(&setlist_item.key))
    }

    pub fn titles(&self) -> Vec<String> {
        self.songs.iter().map(|song| song.title.clone()).collect()
    }
}
