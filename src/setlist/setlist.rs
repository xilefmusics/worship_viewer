use serde::{Deserialize, Serialize};

use std::fs;
use std::path::PathBuf;

use super::{Error, SetlistItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setlist {
    pub title: String,
    pub path: Option<PathBuf>,
    items: Vec<SetlistItem>,
}

impl Setlist {
    pub fn new(title: String, items: Vec<SetlistItem>) -> Self {
        let path = None;
        Self { title, path, items }
    }

    pub fn load(path: PathBuf) -> Result<Self, Error> {
        // get title from filename
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

        // parse setlist items
        let items = fs::read_to_string(&path)?
            .lines()
            .map(|content| {
                let mut iter = content.split(";");
                let title = iter
                    .next()
                    .ok_or(Error::ParseSetlist("no title".to_string()))
                    .map(|title| title.to_string())?;
                let key = iter
                    .next()
                    .ok_or(Error::ParseSetlist("no key".to_string()))
                    .map(|key| key.to_string())?;
                Ok(SetlistItem { title, key })
            })
            .collect::<Result<Vec<SetlistItem>, Error>>()?;

        let path = Some(path);
        Ok(Self { title, path, items })
    }

    pub fn items(&self) -> Vec<SetlistItem> {
        self.items.clone()
    }

    pub fn titles(&self) -> Vec<String> {
        self.items.iter().map(|item| item.title.clone()).collect()
    }

    pub fn ref_titles(&self) -> Vec<&str> {
        self.items.iter().map(|item| item.title.as_str()).collect()
    }
}
