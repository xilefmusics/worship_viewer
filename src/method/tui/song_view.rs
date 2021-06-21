use pancurses::Window;

use std::sync::Arc;

use crate::setlist::SetlistItem;
use crate::song::SongPool;
use crate::tui::SongDisplay;

use super::super::Error;

pub struct SongView {
    setlist_item: Option<SetlistItem>,
    song_pool: Arc<SongPool>,
    key: String,
    song_display: SongDisplay,
}

impl SongView {
    pub fn new(
        parent_window: &Window,
        x_start: i32,
        song_pool: Arc<SongPool>,
    ) -> Result<Self, Error> {
        let song_display = SongDisplay::new(
            parent_window.get_max_y(),
            parent_window.get_max_x() - x_start,
            0,
            x_start,
            parent_window,
        )?;
        let setlist_item = None;
        let key = String::from("Self");
        Ok(Self {
            song_display,
            setlist_item,
            song_pool,
            key,
        })
    }

    pub fn set_key(&mut self, key: &str) {
        self.key = key.to_string();
        self.render()
    }
    pub fn set_b(&mut self) {
        if self.key != "Self" {
            self.key = self
                .key
                .chars()
                .take(1)
                .chain("b".chars())
                .collect::<String>();
            self.render()
        }
    }

    pub fn set_sharp(&mut self) {
        if self.key != "Self" {
            self.key = self
                .key
                .chars()
                .take(1)
                .chain("#".chars())
                .collect::<String>();
            self.render()
        }
    }

    pub fn render(&self) {
        if let Some(mut setlist_item) = self.setlist_item.clone() {
            if self.key != "Self" {
                setlist_item.key = self.key.to_string();
            }
            if let Ok(Some(song)) = self.song_pool.get(&setlist_item) {
                self.song_display.display(song);
            } else {
                self.song_display.text("Error: No song found");
            }
        } else {
            self.song_display.text("Error: No song found");
        }
    }

    pub fn load_setlist_item(&mut self, setlist_item: SetlistItem) {
        self.setlist_item = Some(setlist_item);
        self.render();
    }

    pub fn get_setlist_item(&self) -> Result<SetlistItem, Error> {
        if self.setlist_item.is_none() {
            return Err(Error::NoSong);
        }
        let mut setlist_item = self
            .setlist_item
            .clone()
            .expect("None case is already covered");
        if self.key != "Self" {
            setlist_item.key = self.key.to_string();
        }
        Ok(setlist_item)
    }
}
