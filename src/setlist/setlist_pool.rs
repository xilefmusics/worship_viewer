use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use crate::song::SongPool;

use super::{Error, Setlist, SetlistItem};

pub struct SetlistPool {
    setlists: Vec<Setlist>,
    song_pool: Rc<SongPool>,
}

impl SetlistPool {
    pub fn new(path: &PathBuf, song_pool: Rc<SongPool>) -> Result<Self, Error> {
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
        Ok(Self {
            setlists,
            song_pool,
        })
    }

    pub fn titles(&self) -> Vec<String> {
        std::iter::once("All Songs".to_string())
            .chain(self.setlists.iter().map(|setlist| setlist.title.clone()))
            .collect()
    }

    pub fn true_titles(&self) -> Vec<String> {
        self.setlists
            .iter()
            .map(|setlist| setlist.title.clone())
            .collect()
    }

    pub fn all_songs(&self) -> Setlist {
        let items = self
            .song_pool
            .titles()
            .into_iter()
            .map(|title| SetlistItem {
                title,
                key: "Self".to_string(),
            })
            .collect::<Vec<SetlistItem>>();
        Setlist::new("All Songs".to_string(), items)
    }

    pub fn get(&self, title: String) -> Option<Setlist> {
        match title.as_str() {
            "All Songs" => Some(self.all_songs()),
            _ => self
                .setlists
                .iter()
                .find(|setlist| setlist.title == title)
                .map(|setlist| setlist.clone()),
        }
    }

    pub fn get_first(&self) -> Option<Setlist> {
        self.setlists.get(0).map(|setlist| setlist.clone())
    }
}
