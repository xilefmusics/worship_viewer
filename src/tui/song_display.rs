use pancurses::Window;
use pancurses::{A_BOLD, A_NORMAL};

use crate::song::Song;

use super::Error;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    DefaultOnly,
    TranslationOnly,
    Both,
}

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

    pub fn display(&self, song: Song, mode: Mode) {
        let (show_default, show_translation_text, show_translation_chord, colors) = match mode {
            Mode::Both => (true, true, false, (1, 2, 3, 5)),
            Mode::DefaultOnly => (true, false, false, (1, 2, 3, 5)),
            Mode::TranslationOnly => (false, true, true, (1, 2, 2, 5)),
        };

        self.clear();
        let mut idx = 0;
        for section in song.sections {
            if let Some(keyword) = section.keyword {
                self.window.attrset(A_BOLD);
                self.window.color_set(colors.0);
                self.window.mvprintw(idx + 1, 2, keyword);
                self.window.color_set(0);
                self.window.attrset(A_NORMAL);
                idx += 1;
            }
            for line in section.lines {
                if show_default {
                    if let Some(chord) = line.chord {
                        self.window.attrset(A_BOLD);
                        self.window.color_set(colors.1);
                        self.window.mvprintw(idx + 1, 4, chord);
                        self.window.color_set(0);
                        self.window.attrset(A_NORMAL);
                        idx += 1;
                    }
                    if let Some(text) = line.text {
                        self.window.color_set(colors.1);
                        self.window.mvprintw(idx + 1, 4, text);
                        self.window.color_set(0);
                        idx += 1;
                    }
                }
                if show_translation_chord {
                    if let Some(translation_chord) = line.translation_chord {
                        self.window.attrset(A_BOLD);
                        self.window.color_set(colors.2);
                        self.window.mvprintw(idx + 1, 4, translation_chord);
                        self.window.color_set(0);
                        self.window.attrset(A_NORMAL);
                        idx += 1;
                    }
                }
                if show_translation_text {
                    if let Some(translation_text) = line.translation_text {
                        self.window.color_set(colors.2);
                        self.window.mvprintw(idx + 1, 4, translation_text);
                        self.window.color_set(0);
                        idx += 1;
                    }
                }
                if let Some(comment) = line.comment {
                    self.window.attrset(A_BOLD);
                    self.window.color_set(colors.3);
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
