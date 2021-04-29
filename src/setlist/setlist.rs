use std::fs;
use std::path::PathBuf;

use super::Error;

use crate::song::Song;

#[derive(Debug, Clone)]
pub struct Setlist {
    pub title: String,
    pub path: PathBuf,
}

impl Setlist {
    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let extension = path.extension().and_then(|name| name.to_str());
        let mut title = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(Error::IO("Can't parse title".to_string()))?
            .to_string();
        if let Some(extension) = extension {
            if let Some(pos) = title.find(extension) {
                title = title[..pos - 1].to_string();
            }
        }
        Ok(Self { title, path })
    }

    // TODO remove
    pub fn songs(&self, songs: &Vec<Song>) -> Result<Vec<Song>, Error> {
        fs::read_to_string(&self.path)?
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

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        fs::read_to_string(&self.path)?
            .lines()
            .map(|content| {
                content
                    .split(";")
                    .next()
                    .ok_or(Error::ParseSetlist("no title".to_string()))
                    .map(|title| title.to_string())
            })
            .collect()
    }
}
