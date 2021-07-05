use pancurses::Window;
use pancurses::{A_BOLD, A_NORMAL};

use crate::song::Song;

use super::Error;

pub struct SongDisplay {
    window: Window,
}

impl SongDisplay {
    pub fn new(
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
        parent: &Window,
    ) -> Result<Self, Error> {
        let window = parent.subwin(nlines, ncols, begy, begx)?;
        Ok(Self { window })
    }

    pub fn render(&self) {
        self.clear();
    }

    pub fn clear(&self) {
        self.window.clear();
        self.window.draw_box(0, 0);
    }

    pub fn text(&self, text: &str) {
        self.clear();
        self.window.mvprintw(1, 2, text);
        self.window.refresh();
    }

    pub fn display(&self, song: Song) {
        self.clear();
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
                if let Some(translation) = line.translation_text {
                    self.window.color_set(3);
                    self.window.mvprintw(idx + 1, 4, translation);
                    self.window.color_set(0);
                    idx += 1;
                }
                if let Some(comment) = line.comment {
                    self.window.attrset(A_BOLD);
                    self.window.color_set(5);
                    self.window.mvprintw(idx + 1, 4, comment);
                    self.window.color_set(0);
                    self.window.attrset(A_NORMAL);
                    idx += 1;
                }
            }
            idx += 1;
        }
        self.window.refresh();
    }
}
