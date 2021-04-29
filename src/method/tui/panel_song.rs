use pancurses::{Input, Window};

use crate::song::Song;
use crate::tui::List;

use super::super::Error;
use super::SongView;

pub struct PanelSong {
    sidebar: List<String>,
    song_view: SongView,
    songs: Vec<Song>,
}

impl PanelSong {
    pub fn new(window: &Window, width: i32, songs: Vec<Song>) -> Result<Self, Error> {
        let titles = songs
            .iter()
            .map(|song| song.title.clone())
            .collect::<Vec<String>>();
        let sidebar = List::new(window.get_max_y(), width, 0, 0, window, titles)?;
        let song_view = SongView::new(window, width)?;
        let first_song = songs[0].clone();
        let mut s = Self {
            sidebar,
            song_view,
            songs,
        };
        s.song_view.load_song(first_song)?;
        s.render()?;
        Ok(s)
    }

    pub fn load_selected_song(&mut self) -> Result<(), Error> {
        if let Some(title) = self.sidebar.selected_item() {
            let song = self
                .songs
                .iter()
                .find(|song| song.title == title)
                .ok_or(Error::SongNotFound(title))?
                .clone();
            self.song_view.load_song(song)?;
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<(), Error> {
        self.sidebar.next();
        self.load_selected_song()?;
        Ok(())
    }

    pub fn prev(&mut self) -> Result<(), Error> {
        self.sidebar.prev();
        self.load_selected_song()?;
        Ok(())
    }

    pub fn render(&self) -> Result<(), Error> {
        self.sidebar.render();
        self.song_view.render()?;
        Ok(())
    }

    pub fn handle_input(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::KeyDown) | Some(Input::Character('j')) | Some(Input::Character(' ')) => {
                self.next()?;
            }
            Some(Input::KeyUp) | Some(Input::Character('k')) => {
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
                self.sidebar.isearch(false)?;
                self.load_selected_song()?;
            }
            Some(Input::Character('?')) => {
                self.sidebar.isearch(true)?;
                self.load_selected_song()?;
            }
            _ => (),
        };
        Ok(())
    }
}
