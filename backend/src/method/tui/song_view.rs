use pancurses::Window;

use std::sync::Arc;

use crate::setlist::SetlistItem;
use crate::song::SongPool;
use crate::tui::{SongDisplay, SongDisplayMode};

use super::super::Error;

pub struct SongView {
    setlist_item: Option<SetlistItem>,
    song_pool: Arc<SongPool>,
    key: String,
    mode: SongDisplayMode,
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
        let mode = SongDisplayMode::DefaultOnly;
        Ok(Self {
            song_display,
            setlist_item,
            song_pool,
            key,
            mode,
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

    pub fn set_mode(&mut self, mode: SongDisplayMode) {
        self.mode = mode;
        self.render()
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SongDisplayMode::DefaultOnly => SongDisplayMode::TranslationOnly,
            SongDisplayMode::TranslationOnly => SongDisplayMode::DefaultOnly,
            SongDisplayMode::Both => SongDisplayMode::DefaultOnly,
        };
        self.render()
    }

    pub fn render(&self) {
        if let Some(mut setlist_item) = self.setlist_item.clone() {
            if self.key != "Self" {
                setlist_item.key = self.key.to_string();
            }
            let mut mode = self.mode; // TODO mode from setlist_item
            if let Ok(Some(song)) = self.song_pool.get(&setlist_item) {
                if !song.has_translation() {
                    mode = SongDisplayMode::DefaultOnly;
                }
                self.song_display.display(song, mode);
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
