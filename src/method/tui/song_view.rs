use pancurses::Window;
use pancurses::{A_BOLD, A_NORMAL};

use super::super::super::line::{IterExtToMulti, IterExtTranspose, Multiline};
use super::super::super::song::Song;

use super::super::Error;

pub struct SongView {
    window: Window,
    song: Option<Song>,
    key: String,
}

impl SongView {
    pub fn new(parent_window: &Window, x_start: i32) -> Result<Self, Error> {
        let window = parent_window
            .subwin(
                parent_window.get_max_y(),
                parent_window.get_max_x() - x_start,
                0,
                x_start,
            )
            .map_err(|_| Error::Tui)?;
        let song = None;
        let key = String::from("Self");
        window.draw_box(0, 0);
        window.keypad(true);
        Ok(Self { window, song, key })
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
        if let Some(song) = &self.song {
            let mut idx = 0;
            let mut first_section = true;
            let key = match self.key.as_str() {
                "Self" => song.key.as_str(),
                key => key,
            };
            song.lines()
                .into_iter()
                .transpose(key)
                .to_multi_flatten()
                .for_each(|line| match line {
                    Multiline::Keyword(keyword) => {
                        if first_section {
                            first_section = false;
                        } else {
                            idx += 1;
                        }
                        self.window.attrset(A_BOLD);
                        self.window.color_set(1);
                        self.window.mvprintw(idx + 1, 2, keyword);
                        self.window.color_set(0);
                        self.window.attrset(A_NORMAL);
                        idx += 1;
                    }

                    Multiline::Chord(chord) => {
                        self.window.attrset(A_BOLD);
                        self.window.color_set(2);
                        self.window.mvprintw(idx + 1, 4, chord);
                        self.window.color_set(0);
                        self.window.attrset(A_NORMAL);
                        idx += 1;
                    }
                    Multiline::Text(text) => {
                        self.window.color_set(2);
                        self.window.mvprintw(idx + 1, 4, text);
                        self.window.color_set(0);
                        idx += 1;
                    }

                    Multiline::Translation(translation) => {
                        self.window.color_set(3);
                        self.window.mvprintw(idx + 1, 4, translation);
                        self.window.color_set(0);
                        idx += 1;
                    }
                });
            self.window.refresh();
            return Ok(());
        }
        Err(Error::NoSong)
    }

    pub fn load_song(&mut self, song: Song) -> Result<(), Error> {
        self.song = Some(song);
        self.render()?;
        Ok(())
    }
}
