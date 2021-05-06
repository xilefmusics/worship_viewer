use reqwest::{self, StatusCode};

use super::super::{Error, Song};
use crate::setlist::SetlistItem;

pub struct SongPoolRemote {
    url: String,
}

impl SongPoolRemote {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn get(&self, setlist_item: &SetlistItem) -> Result<Option<Song>, Error> {
        let url = format!(
            "{}/song/{}/{}",
            self.url, setlist_item.title, setlist_item.key
        );
        let res = reqwest::blocking::get(&url)?;
        let not_found = StatusCode::from_u16(404).expect("404 is a valid status code");
        if res.status() == not_found {
            return Ok(None);
        }
        Ok(res.json()?)
    }

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        let url = format!("{}/song_titles", self.url);
        Ok(reqwest::blocking::get(&url)?.json()?)
    }
}
