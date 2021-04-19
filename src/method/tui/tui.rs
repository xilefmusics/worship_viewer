use pancurses::{endwin, initscr};
use pancurses::{Input, Window};
use pancurses::{COLOR_CYAN, COLOR_GREEN, COLOR_RED, COLOR_WHITE};

use std::env;

use super::super::Error;

use super::{Config, Sidebar, Song, SongView};

pub fn tui(args: env::Args) -> Result<(), Error> {
    let window = initscr();
    let result = tui_inner(args, &window);
    endwin();
    result
}

fn tui_inner(args: env::Args, window: &Window) -> Result<(), Error> {
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
            Some(Input::KeyDown) | Some(Input::Character('j')) => {
                song_view.load_song(sidebar.next())?;
            }
            Some(Input::KeyUp) | Some(Input::Character('k')) => {
                song_view.load_song(sidebar.prev())?;
            }
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
    }

    Ok(())
}
