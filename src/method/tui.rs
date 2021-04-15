extern crate pancurses;
use pancurses::{initscr, endwin};
use pancurses::{Window, Input};
use pancurses::{A_NORMAL, A_BOLD};
use pancurses::{COLOR_RED, COLOR_GREEN, COLOR_CYAN, COLOR_WHITE};

use std::{env, fs};
use std::path::PathBuf;

use super::Error;
use super::super::line::{IterExtToWp, IterExtToMulti, IterExtTranspose, Multiline};
use super::super::line::wp;


struct Config {
    folder: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let folder = match args.next() {
            Some(folder) => folder,
            None => String::from("."),
        };
        Ok(Self{folder})
    }
}

#[derive(Debug, Clone)]
struct Song {
    title: String,
    path: PathBuf,
}

impl Song {
    fn load_vec(config: &Config) -> Result<Vec<Self>, Error>{
        let mut songs = fs::read_dir(&config.folder)
            .map_err(|_| Error::IO)?
            .map(|res| res.map(|e| e.path()))
            .map(|path| {
                let path = path.map_err(|_| Error::IO)?.clone();
                let line = fs::read_to_string(&path)
                    .map_err(|_| Error::IO)?
                    .lines()
                    .to_wp()
                    .find(|line| {
                        match line {
                            wp::Line::Directive((key, _)) => match key.as_str() {
                                "title" => true,
                                _ => false,
                            },
                            _ => false,
                        }
                    });

                let title = match line {
                    Some(wp::Line::Directive((_, title))) => title,
                    _ => String::new(),
                };
                Ok(Song{title, path})
        }).collect::<Result<Vec<Song>, Error>>()?;

        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(songs)
    }
}

struct Sidebar {
    current_index: usize,
    window: Window,
    songs: Vec<Song>,
    width: usize,
}

impl Sidebar {
    fn new(parent_window: &Window, width: i32, songs: Vec<Song>) -> Result<Self, Error> {
        let current_index = 0;
        let window = parent_window.subwin(parent_window.get_max_y(), width, 0, 0).map_err(|_| Error::Tui)?;
        let width = width as usize;
        window.draw_box(0, 0);
        window.keypad(true);
        Ok(Self{current_index, window, songs, width})
    }

    fn render(&self) {
        for (idx, Song{path , title}) in self.songs.iter().enumerate() {
            if idx == self.current_index {
                self.window.attrset(A_BOLD);
                self.window.color_set(4);
            }
            self.window.mvprintw((idx+1) as i32, 1, " ");
            self.window.printw(title);
            self.window.printw(&std::iter::repeat(" ").take(self.width-3-title.chars().count()).collect::<String>());

            if idx == self.current_index {
                self.window.attrset(A_NORMAL);
                self.window.color_set(0);
            }
            self.window.refresh();
        }
    }

    fn get_song(&self) -> Song {
        self.songs[self.current_index].clone()
    }

    fn next(&mut self) -> Song {
        self.current_index = (self.current_index + 1).rem_euclid(self.songs.len());
        self.render();
        self.get_song()
    }

    fn prev(&mut self) -> Song {
        self.current_index = (self.current_index as i32 - 1).rem_euclid(self.songs.len() as i32) as usize;
        self.render();
        self.get_song()
    }

}

struct SongView {
    window: Window,
    song: Option<Song>,
    key: String,
}

impl SongView {
    fn new(parent_window: &Window, x_start: i32) -> Result<Self, Error> {
        let window = parent_window.subwin(parent_window.get_max_y(), parent_window.get_max_x()-x_start, 0, x_start).map_err(|_| Error::Tui)?; let song = None;
        let key = String::from("Self");
        window.draw_box(0, 0);
        window.keypad(true);
        Ok(Self{window, song, key})
    }

    fn set_key(&mut self, key: &str) -> Result<(), Error> {
        self.key = key.to_string();
        self.render()
    }
    fn set_b(&mut self) -> Result<(), Error> {
        if self.key == "Self" {
            return Ok(());
        }
        self.key = self.key.chars().take(1).chain("b".chars()).collect::<String>();
        self.render()
    }

    fn set_sharp(&mut self) -> Result<(), Error> {
        if self.key == "Self" {
            return Ok(());
        }
        self.key = self.key.chars().take(1).chain("#".chars()).collect::<String>();
        self.render()
    }

    fn render(&self) -> Result<(), Error> {
        self.window.clear();
        self.window.draw_box(0, 0);
        if let Some(song) = &self.song {
            let mut idx = 0;
            let mut first_section = true;
            fs::read_to_string(&song.path)
                .map_err(|_| {
                    Error::IO
                })?
                .lines()
                .to_wp()
                .transpose(&self.key)
                .to_multi()
                .for_each(|line| {
                    match line {
                        Multiline::Keyword(keyword) => {
                            if first_section {
                                first_section = false;
                            } else {
                                idx += 1;
                            }
                            self.window.attrset(A_BOLD);
                            self.window.color_set(1);
                            self.window.mvprintw(idx+1, 2, keyword);
                            self.window.color_set(0);
                            self.window.attrset(A_NORMAL);
                            idx += 1;
                        },

                        Multiline::Chord(chord) => {
                            self.window.attrset(A_BOLD);
                            self.window.color_set(2);
                            self.window.mvprintw(idx+1, 4, chord);
                            self.window.color_set(0);
                            self.window.attrset(A_NORMAL);
                            idx += 1;
                        },
                        Multiline::Text(text) => {
                            self.window.color_set(2);
                            self.window.mvprintw(idx+1, 4, text);
                            self.window.color_set(0);
                            idx += 1;
                        },

                        Multiline::Translation(translation) => {
                            self.window.color_set(3);
                            self.window.mvprintw(idx+1, 4, translation);
                            self.window.color_set(0);
                            idx += 1;
                        },
                    }
                });
            self.window.refresh();
            return Ok(());
        }
        Err(Error::NoSong)
    }

    fn load_song(&mut self, song: Song) -> Result<(), Error> {
        self.song = Some(song);
        self.render()?;
        Ok(())
    }
}

pub fn tui(args: env::Args) -> Result<(), Error> {
    let window = initscr();
    let result = tui_inner(args, &window);
    endwin();
    result
}


pub fn tui_inner(args: env::Args, window: &Window) -> Result<(), Error> {
    let config = Config::new(args)?;
    let songs = Song::load_vec(&config)?;

    pancurses::noecho();
    pancurses::curs_set(0);
    pancurses::start_color();
    pancurses::use_default_colors();
    pancurses::init_pair(0, -1, -1);
    pancurses::init_pair(1, COLOR_RED, -1);
    pancurses::init_pair(2, COLOR_GREEN, -1);
    pancurses::init_pair(3, COLOR_CYAN, -1);
    pancurses::init_pair(4, COLOR_WHITE, COLOR_GREEN);

    let mut song_view = SongView::new(&window, 40)?;
    let mut sidebar = Sidebar::new(&window, 40, songs)?;

    sidebar.render();
    song_view.load_song(sidebar.get_song())?;

    loop {
        match window.getch() {
            Some(Input::KeyDown)|Some(Input::Character('j')) => {
                song_view.load_song(sidebar.next())?;
            },
            Some(Input::KeyUp)|Some(Input::Character('k')) => {
                song_view.load_song(sidebar.prev())?;
            },
            Some(Input::Character('q')) => break,
            Some(Input::Character('A')) => song_view.set_key("A")?,
            Some(Input::Character('B')) => song_view.set_key("B")?,
            Some(Input::Character('C')) => song_view.set_key("C")?,
            Some(Input::Character('D')) => song_view.set_key("D")?,
            Some(Input::Character('E')) => song_view.set_key("E")?,
            Some(Input::Character('F')) => song_view.set_key("F")?,
            Some(Input::Character('G')) => song_view.set_key("G")?,
            Some(Input::Character('b')) => song_view.set_b()?,
            Some(Input::Character('#')) => song_view.set_sharp()?,
            Some(Input::Character('r')) => song_view.set_key("Self")?,
            _ => (),
        }
    };

    Ok(())
}
