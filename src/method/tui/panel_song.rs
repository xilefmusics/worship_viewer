use pancurses::{Input, Window};

use super::super::Error;
use super::{Sidebar, Song, SongView};

pub struct PanelSong {
    sidebar: Sidebar,
    song_view: SongView,
}

impl PanelSong {
    pub fn new(window: &Window, width: i32, songs: Vec<Song>) -> Result<Self, Error> {
        let sidebar = Sidebar::new(window, width, songs)?;
        let song_view = SongView::new(window, width)?;
        let mut s = Self { sidebar, song_view };
        s.song_view.load_song(s.sidebar.get_song())?;
        s.render()?;
        Ok(s)
    }

    pub fn next(&mut self) -> Result<(), Error> {
        let song = self.sidebar.next();
        self.song_view.load_song(song)?;
        Ok(())
    }

    pub fn prev(&mut self) -> Result<(), Error> {
        let song = self.sidebar.prev();
        self.song_view.load_song(song)?;
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
            _ => (),
        };
        Ok(())
    }
}
