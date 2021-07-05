use pancurses::{Input, Window};

use std::cmp;
use std::fmt;
use std::sync::Arc;

use crate::method::Error;
use crate::song::import::UltimateGuitarSearchResult as SearchResult;
use crate::song::import::{ultimate_guitar_search, ultimate_guitar_to_song};
use crate::song::SongPool;
use crate::tui::{InputBox, List, SongDisplay, SongDisplayMode};

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.song_name, self.artist_name)
    }
}

impl cmp::Ord for SearchResult {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.song_name.cmp(&other.song_name)
    }
}

impl cmp::PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl cmp::PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.song_name == other.song_name && self.artist_name == other.artist_name
    }
}

impl cmp::Eq for SearchResult {}

pub struct PanelImport {
    song_pool: Arc<SongPool>,
    list: List<SearchResult>,
    input: InputBox,
    song_display: SongDisplay,
}

impl PanelImport {
    pub fn new(window: &Window, width: i32, song_pool: Arc<SongPool>) -> Result<Self, Error> {
        let list = List::new(window.get_max_y(), width, 0, 0, window, vec![])?;
        let input = InputBox::new(3, width - 2, window.get_max_y() - 4, 1, window)?;
        let song_display = SongDisplay::new(
            window.get_max_y(),
            window.get_max_x() - width,
            0,
            width,
            window,
        )?;
        Ok(Self {
            song_pool,
            list,
            input,
            song_display,
        })
    }

    pub fn render(&self) {
        self.list.render();
        self.song_display.render();
    }

    pub fn search(&mut self) {
        if let Some(search_str) = self.input.input() {
            match ultimate_guitar_search(&search_str) {
                Ok(search_results) => {
                    self.list.change_items(search_results);
                    self.load_selected_song();
                }
                Err(_) => self.song_display.text("Error while searching"),
            }
        }
    }

    fn load_selected_song(&mut self) {
        if let Some(item) = self.list.selected_item() {
            match ultimate_guitar_to_song(&item.url) {
                Ok(song) => self
                    .song_display
                    .display(song, SongDisplayMode::DefaultOnly),
                Err(_) => self.song_display.text("Error while parsing song"),
            }
        }
    }

    fn write_selected_song(&mut self) {
        if let Some(item) = self.list.selected_item() {
            match ultimate_guitar_to_song(&item.url) {
                Ok(song) => {
                    if let Err(_) = self.song_pool.create(song) {
                        self.song_display.text("Error while writing song")
                    } else {
                        self.song_display.text("Wrote song to library")
                    }
                }
                Err(_) => self.song_display.text("Error while parsing song"),
            }
        }
    }

    pub fn handle_input(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('s')) => self.search(),
            Some(Input::Character('w')) => self.write_selected_song(),
            Some(Input::Character('j')) | Some(Input::Character(' ')) => {
                self.list.next();
                self.load_selected_song();
            }
            Some(Input::Character('k')) => {
                self.list.prev();
                self.load_selected_song();
            }
            Some(Input::Character('/')) => {
                self.list.isearch(false)?;
                self.load_selected_song();
            }
            Some(Input::Character('?')) => {
                self.list.isearch(true)?;
                self.load_selected_song();
            }
            input => self.list.handle_input(input)?,
        }
        Ok(())
    }
}
