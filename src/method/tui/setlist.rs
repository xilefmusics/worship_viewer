use std::fs;
use std::path::PathBuf;

use super::super::Error;

use crate::song::Song;

#[derive(Debug, Clone)]
pub struct Setlist {
    pub title: String,
    pub path: PathBuf,
}

impl Setlist {
    pub fn _load_all(path: PathBuf) -> Result<Vec<Self>, Error> {
        let mut path = path;
        path.push("setlists");
        let mut setlists = fs::read_dir(path)
            .map_err(|_| Error::IO)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| {
                if let Ok(path) = path {
                    !path.is_dir()
                } else {
                    false
                }
            })
            .map(|path| Self::load(path.map_err(|_| Error::IO)?))
            .collect::<Result<Vec<Self>, Error>>()?;
        setlists.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(setlists)
    }

    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let extension = path.extension().and_then(|name| name.to_str());
        let mut title = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(Error::IO)?
            .to_string();
        if let Some(extension) = extension {
            if let Some(pos) = title.find(extension) {
                title = title[..pos - 1].to_string();
            }
        }
        Ok(Self { title, path })
    }

    pub fn songs(&self, songs: &Vec<Song>) -> Result<Vec<Song>, Error> {
        fs::read_to_string(&self.path)
            .map_err(|_| Error::IO)?
            .lines()
            .map(|content| {
                let mut iter = content.split(";");
                let title = iter
                    .next()
                    .ok_or(Error::ParseSetlist("no title".to_string()))?;
                let key: String = iter
                    .next()
                    .and_then(|key| Some(key.to_string()))
                    .unwrap_or("Self".to_string());
                Ok(songs
                    .iter()
                    .find(|song| song.title == title)
                    .ok_or(Error::ParseSetlist("Song not found".to_string()))?
                    .transpose(key))
            })
            .collect::<Result<Vec<Song>, Error>>()
    }
}
