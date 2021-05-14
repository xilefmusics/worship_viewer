use pancurses::{Input, Window};

use std::sync::Arc;

use crate::setlist::{SetlistItem, SetlistPool};
use crate::song::SongPool;
use crate::tui::List;

use super::super::Error;
use super::SongView;

enum Mode {
    Song,
    Setlist,
}

pub struct PanelSong {
    sidebar_setlist: List<String>,
    sidebar_song: List<SetlistItem>,
    song_view: SongView,
    song_pool: Arc<SongPool>,
    setlist_pool: Arc<SetlistPool>,
    mode: Mode,
}

impl PanelSong {
    pub fn new(
        window: &Window,
        width: i32,
        song_pool: Arc<SongPool>,
        setlist_pool: Arc<SetlistPool>,
    ) -> Result<Self, Error> {
        let sidebar_setlist = List::new(window.get_max_y(), width, 0, 0, window, vec![])?;
        let sidebar_song = List::new(
            window.get_max_y(),
            width,
            0,
            0,
            window,
            setlist_pool.all_songs()?.items(),
        )?;
        let song_view = SongView::new(window, width, Arc::clone(&song_pool))?;
        let mode = Mode::Song;
        let mut s = Self {
            sidebar_setlist,
            sidebar_song,
            song_view,
            song_pool,
            setlist_pool,
            mode,
        };
        s.load_selected_song()?;
        s.render()?;
        Ok(s)
    }

    pub fn load_selected_song(&mut self) -> Result<(), Error> {
        if let Some(setlist_item) = self.sidebar_song.selected_item() {
            self.song_view.load_setlist_item(setlist_item)?;
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<(), Error> {
        self.sidebar_song.next();
        self.load_selected_song()?;
        Ok(())
    }

    pub fn prev(&mut self) -> Result<(), Error> {
        self.sidebar_song.prev();
        self.load_selected_song()?;
        Ok(())
    }

    pub fn render(&self) -> Result<(), Error> {
        match self.mode {
            Mode::Song => self.sidebar_song.render(),
            Mode::Setlist => self.sidebar_setlist.render(),
        }
        self.song_view.render()?;
        Ok(())
    }
    pub fn handle_input(&mut self, input: Option<Input>) -> Result<(), Error> {
        match self.mode {
            Mode::Song => self.handle_input_mode_song(input),
            Mode::Setlist => self.handle_input_mode_setlist(input),
        }
    }

    pub fn select_setlist(&mut self) -> Result<(), Error> {
        self.sidebar_setlist
            .change_items(self.setlist_pool.titles()?);
        self.mode = Mode::Setlist;
        Ok(())
    }

    pub fn load_setlist(&mut self) -> Result<(), Error> {
        if let Some(title) = self.sidebar_setlist.selected_item() {
            if let Some(setlist) = self.setlist_pool.get(title)? {
                self.sidebar_song.change_items(setlist.items());
            }
            self.load_selected_song()?;
            self.mode = Mode::Song;
        }
        Ok(())
    }

    pub fn handle_input_mode_song(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('j')) | Some(Input::Character(' ')) => {
                self.next()?;
            }
            Some(Input::Character('k')) => {
                self.prev()?;
            }
            Some(Input::Character('A')) => self.song_view.set_key("A")?,
            Some(Input::Character('B')) => self.song_view.set_key("B")?,
            Some(Input::Character('C')) => self.song_view.set_key("C")?,
            Some(Input::Character('D')) => self.song_view.set_key("D")?,
            Some(Input::Character('E')) => self.song_view.set_key("E")?,
            Some(Input::Character('F')) => self.song_view.set_key("F")?,
            Some(Input::Character('G')) => self.song_view.set_key("G")?,
            Some(Input::Character('b')) => self.song_view.set_b()?,
            Some(Input::Character('#')) => self.song_view.set_sharp()?,
            Some(Input::Character('r')) => self.song_view.set_key("Self")?,
            Some(Input::Character('/')) => {
                self.sidebar_song.isearch(false)?;
                self.load_selected_song()?;
            }
            Some(Input::Character('?')) => {
                self.sidebar_song.isearch(true)?;
                self.load_selected_song()?;
            }
            Some(Input::Character('\t')) => self.select_setlist()?,
            Some(Input::Character('e')) => {
                if let Some(setlist_item) = self.sidebar_song.selected_item() {
                    self.song_pool.edit(&setlist_item)?;
                    self.song_pool.reload(&setlist_item)?;
                    self.render()?;
                }
            }
            _ => (),
        };
        Ok(())
    }

    pub fn handle_input_mode_setlist(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character(' ')) | Some(Input::Character('\t')) => self.load_setlist()?,
            input => self.sidebar_setlist.handle_input(input)?,
        }
        Ok(())
    }
}
