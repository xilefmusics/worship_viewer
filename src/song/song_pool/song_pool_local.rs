use std::env::var;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::super::{Error, Song, SongIntern};
use crate::setlist::SetlistItem;

pub struct SongPoolLocal {
    songs: Arc<Mutex<Vec<SongIntern>>>,
}

impl SongPoolLocal {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut songs = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| path.is_ok() && !path.as_ref().unwrap().is_dir())
            .map(|path| SongIntern::load(path?.clone()))
            .collect::<Result<Vec<SongIntern>, Error>>()?;
        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        let songs = Arc::new(Mutex::new(songs));
        Ok(Self { songs })
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Option<Song> {
        self.songs
            .lock()
            .unwrap()
            .iter()
            .find(|song| song.title == setlist_item.title)
            .map(|song| song.to_section_song(&setlist_item.key))
    }

    pub fn titles(&self) -> Vec<String> {
        self.songs
            .lock()
            .unwrap()
            .iter()
            .map(|song| song.title.clone())
            .collect()
    }

    pub fn reload(&self, setlist_item: &SetlistItem) -> Result<(), Error> {
        let mut songs = self.songs.lock().unwrap();
        let (idx, _) = songs
            .iter()
            .enumerate()
            .find(|(_, song)| song.title == setlist_item.title)
            .ok_or(Error::Other("Song not found".to_string()))?;

        let path = songs[idx].path.clone();
        let song = SongIntern::load(path)?;
        songs[idx] = song;
        Ok(())
    }

    pub fn edit(&self, setlist_item: &SetlistItem) -> Result<(), Error> {
        Command::new(var("EDITOR").map_err(|_| Error::Other("No editor".to_string()))?)
            .arg(
                self.songs
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|song| song.title == setlist_item.title)
                    .ok_or(Error::Other("No song".to_string()))?
                    .path
                    .clone(),
            )
            .status()?;
        Ok(())
    }
}
