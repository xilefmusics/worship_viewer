use std::fs;
use std::path::PathBuf;

use super::{Error, Setlist};

pub struct SetlistPool {
    setlists: Vec<Setlist>,
}

impl SetlistPool {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut setlists = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| {
                if let Ok(path) = path {
                    !path.is_dir()
                } else {
                    false
                }
            })
            .map(|path| Setlist::load(path?))
            .collect::<Result<Vec<Setlist>, Error>>()?;
        setlists.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(Self { setlists })
    }

    pub fn titles(&self) -> Vec<String> {
        self.setlists
            .iter()
            .map(|setlist| setlist.title.clone())
            .collect()
    }

    pub fn get(&self, title: String) -> Result<Setlist, Error> {
        self.setlists
            .iter()
            .find(|setlist| setlist.title == title)
            .ok_or(Error::SetlistNotFound(title))
            .map(|setlist| setlist.clone())
    }
}
