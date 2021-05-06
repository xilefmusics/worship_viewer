use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::song::SongPool;

use super::{Error, Setlist, SetlistItem};

pub struct SetlistPoolLocal {
    setlists: Arc<Mutex<Vec<Setlist>>>,
    song_pool: Arc<SongPool>,
}

impl SetlistPoolLocal {
    pub fn new(path: &PathBuf, song_pool: Arc<SongPool>) -> Result<Self, Error> {
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
        let setlists = Arc::new(Mutex::new(setlists));
        Ok(Self {
            setlists,
            song_pool,
        })
    }

    pub fn titles(&self) -> Vec<String> {
        std::iter::once("All Songs".to_string())
            .chain(
                self.setlists
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|setlist| setlist.title.clone()),
            )
            .collect()
    }

    pub fn true_titles(&self) -> Vec<String> {
        self.setlists
            .lock()
            .unwrap()
            .iter()
            .map(|setlist| setlist.title.clone())
            .collect()
    }

    pub fn all_songs(&self) -> Result<Setlist, Error> {
        let items = self
            .song_pool
            .titles()?
            .into_iter()
            .map(|title| SetlistItem {
                title,
                key: "Self".to_string(),
            })
            .collect::<Vec<SetlistItem>>();
        Ok(Setlist::new("All Songs".to_string(), items))
    }

    pub fn get(&self, title: String) -> Result<Option<Setlist>, Error> {
        Ok(match title.as_str() {
            "All Songs" => Some(self.all_songs()?),
            _ => self
                .setlists
                .lock()
                .unwrap()
                .iter()
                .find(|setlist| setlist.title == title)
                .map(|setlist| setlist.clone()),
        })
    }

    pub fn get_first(&self) -> Option<Setlist> {
        self.setlists
            .lock()
            .unwrap()
            .get(0)
            .map(|setlist| setlist.clone())
    }

    pub fn update_setlist(&self, setlist: Setlist) -> Result<(), Error> {
        let mut setlists = self.setlists.lock().unwrap();
        if let Some((idx, _)) = setlists
            .iter()
            .enumerate()
            .find(|(_, sl)| sl.title == setlist.title)
        {
            setlists[idx] = setlist;
        } else {
            setlists.push(setlist);
            setlists.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        Ok(())
    }
}
