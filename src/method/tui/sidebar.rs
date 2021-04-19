use pancurses::Window;
use pancurses::{A_BOLD, A_NORMAL};

use super::super::Error;

use super::Song;

pub struct Sidebar {
    current_index: usize,
    window: Window,
    songs: Vec<Song>,
    width: usize,
}

impl Sidebar {
    pub fn new(parent_window: &Window, width: i32, songs: Vec<Song>) -> Result<Self, Error> {
        let current_index = 0;
        let window = parent_window
            .subwin(parent_window.get_max_y(), width, 0, 0)
            .map_err(|_| Error::Tui)?;
        let width = width as usize;
        window.draw_box(0, 0);
        window.keypad(true);
        Ok(Self {
            current_index,
            window,
            songs,
            width,
        })
    }

    pub fn render(&self) {
        for (idx, Song { title, .. }) in self.songs.iter().enumerate() {
            if idx == self.current_index {
                self.window.attrset(A_BOLD);
                self.window.color_set(4);
            }
            self.window.mvprintw((idx + 1) as i32, 1, " ");
            self.window.printw(title);
            self.window.printw(
                &std::iter::repeat(" ")
                    .take(self.width - 3 - title.chars().count())
                    .collect::<String>(),
            );

            if idx == self.current_index {
                self.window.attrset(A_NORMAL);
                self.window.color_set(0);
            }
            self.window.refresh();
        }
    }

    pub fn get_song(&self) -> Song {
        self.songs[self.current_index].clone()
    }

    pub fn next(&mut self) -> Song {
        self.current_index = (self.current_index + 1).rem_euclid(self.songs.len());
        self.render();
        self.get_song()
    }

    pub fn prev(&mut self) -> Song {
        self.current_index =
            (self.current_index as i32 - 1).rem_euclid(self.songs.len() as i32) as usize;
        self.render();
        self.get_song()
    }
}
