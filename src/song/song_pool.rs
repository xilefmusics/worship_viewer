use std::fs;
use std::path::PathBuf;

use super::{Error, Song};

pub struct SongPool {
    songs: Vec<Song>,
}

impl SongPool {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut songs = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| path.is_ok() && !path.as_ref().unwrap().is_dir())
            .map(|path| Song::load(path?.clone()))
            .collect::<Result<Vec<Song>, Error>>()?;
        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(Self { songs })
    }

    pub fn get(&self, title: String) -> Result<Song, Error> {
        self.songs
            .iter()
            .find(|song| song.title == title)
            .ok_or(Error::SongNotFound(title))
            .map(|song| song.clone())
    }

    pub fn titles(&self) -> Vec<String> {
        self.songs.iter().map(|song| song.title.clone()).collect()
    }
}
