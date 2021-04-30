use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use crate::song::SongPool;

use super::{Error, Setlist, SetlistItem};

pub struct SetlistPool {
    setlists: RefCell<Vec<Setlist>>,
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
        let setlists = RefCell::new(setlists);
        Ok(Self {
            setlists,
            song_pool,
        })
    }

    pub fn titles(&self) -> Vec<String> {
        std::iter::once("All Songs".to_string())
            .chain(
                self.setlists
                    .borrow()
                    .iter()
                    .map(|setlist| setlist.title.clone()),
            )
            .collect()
    }

    pub fn true_titles(&self) -> Vec<String> {
        self.setlists
            .borrow()
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
                .borrow()
                .iter()
                .find(|setlist| setlist.title == title)
                .map(|setlist| setlist.clone()),
        }
    }

    pub fn get_first(&self) -> Option<Setlist> {
        self.setlists.borrow().get(0).map(|setlist| setlist.clone())
    }

    pub fn update_setlist(&self, setlist: Setlist) -> Result<(), Error> {
        let mut setlists = self.setlists.borrow_mut();
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
