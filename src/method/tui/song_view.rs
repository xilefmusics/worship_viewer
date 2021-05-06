use pancurses::Window;
use pancurses::{A_BOLD, A_NORMAL};

use std::sync::Arc;

use crate::setlist::SetlistItem;
use crate::song::SongPool;

use super::super::Error;

pub struct SongView {
    window: Window,
    setlist_item: Option<SetlistItem>,
    song_pool: Arc<SongPool>,
    key: String,
}

impl SongView {
    pub fn new(
        parent_window: &Window,
        x_start: i32,
        song_pool: Arc<SongPool>,
    ) -> Result<Self, Error> {
        let window = parent_window.subwin(
            parent_window.get_max_y(),
            parent_window.get_max_x() - x_start,
            0,
            x_start,
        )?;
        let setlist_item = None;
        let key = String::from("Self");
        window.draw_box(0, 0);
        window.keypad(true);
        Ok(Self {
            window,
            setlist_item,
            song_pool,
            key,
        })
    }

    pub fn set_key(&mut self, key: &str) -> Result<(), Error> {
        self.key = key.to_string();
        self.render()
    }
    pub fn set_b(&mut self) -> Result<(), Error> {
        if self.key == "Self" {
            return Ok(());
        }
        self.key = self
            .key
            .chars()
            .take(1)
            .chain("b".chars())
            .collect::<String>();
        self.render()
    }

    pub fn set_sharp(&mut self) -> Result<(), Error> {
        if self.key == "Self" {
            return Ok(());
        }
        self.key = self
            .key
            .chars()
            .take(1)
            .chain("#".chars())
            .collect::<String>();
        self.render()
    }

    pub fn render(&self) -> Result<(), Error> {
        self.window.clear();
        self.window.draw_box(0, 0);
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
        let song = match self.song_pool.get(&setlist_item) {
            Ok(Some(song)) => song,
            _ => return Ok(()),
        };

        let mut idx = 0;
        for section in song.sections {
            if let Some(keyword) = section.keyword {
                self.window.attrset(A_BOLD);
                self.window.color_set(1);
                self.window.mvprintw(idx + 1, 2, keyword);
                self.window.color_set(0);
                self.window.attrset(A_NORMAL);
                idx += 1;
            }
            for line in section.lines {
                if let Some(chord) = line.chord {
                    self.window.attrset(A_BOLD);
                    self.window.color_set(2);
                    self.window.mvprintw(idx + 1, 4, chord);
                    self.window.color_set(0);
                    self.window.attrset(A_NORMAL);
                    idx += 1;
                }
                if let Some(text) = line.text {
                    self.window.color_set(2);
                    self.window.mvprintw(idx + 1, 4, text);
                    self.window.color_set(0);
                    idx += 1;
                }
                if let Some(translation) = line.translation {
                    self.window.color_set(3);
                    self.window.mvprintw(idx + 1, 4, translation);
                    self.window.color_set(0);
                    idx += 1;
                }
            }
            idx += 1;
        }
        self.window.refresh();
        Ok(())
    }

    pub fn load_setlist_item(&mut self, setlist_item: SetlistItem) -> Result<(), Error> {
        self.setlist_item = Some(setlist_item);
        self.render()?;
        Ok(())
    }
}
