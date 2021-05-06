use reqwest::{self, StatusCode};

use super::{Error, Setlist};

pub struct SetlistPoolRemote {
    url: String,
}

impl SetlistPoolRemote {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn titles(&self) -> Result<Vec<String>, Error> {
        let url = format!("{}/setlist_titles", self.url);
        Ok(reqwest::blocking::get(&url)?.json()?)
    }

    pub fn true_titles(&self) -> Result<Vec<String>, Error> {
        let url = format!("{}/setlist_true_titles", self.url);
        Ok(reqwest::blocking::get(&url)?.json()?)
    }

    pub fn all_songs(&self) -> Result<Setlist, Error> {
        let url = format!("{}/setlist_all_songs", self.url);
        Ok(reqwest::blocking::get(&url)?.json()?)
    }

    pub fn get(&self, title: String) -> Result<Option<Setlist>, Error> {
        let url = format!("{}/setlist/{}", self.url, title);
        let res = reqwest::blocking::get(&url)?;
        let not_found = StatusCode::from_u16(404).expect("404 is a valid status code");
        if res.status() == not_found {
            return Ok(None);
        }
        Ok(res.json()?)
    }

    pub fn get_first(&self) -> Result<Option<Setlist>, Error> {
        let url = format!("{}/setlist_get_first", self.url);
        let res = reqwest::blocking::get(&url)?;
        let not_found = StatusCode::from_u16(404).expect("404 is a valid status code");
        if res.status() == not_found {
            return Ok(None);
        }
        Ok(res.json()?)
    }
}
