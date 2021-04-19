use pancurses::{endwin, initscr};
use pancurses::{Input, Window};
use pancurses::{COLOR_CYAN, COLOR_GREEN, COLOR_RED, COLOR_WHITE};

use std::env;

use super::super::Error;

use super::{Config, PanelSong, Setlist, Song};

pub fn tui(args: env::Args) -> Result<(), Error> {
    let window = initscr();
    let result = tui_inner(args, &window);
    endwin();
    result
}

fn tui_inner(args: env::Args, window: &Window) -> Result<(), Error> {
    let config = Config::new(args)?;

    let songs = match config.setlist_path {
        Some(path) => Setlist::load(path)?.songs(),
        None => Song::load_all(&config.root_path),
    }?;

    pancurses::noecho();
    pancurses::curs_set(0);
    pancurses::start_color();
    pancurses::use_default_colors();
    pancurses::init_pair(0, -1, -1);
    pancurses::init_pair(1, COLOR_RED, -1);
    pancurses::init_pair(2, COLOR_GREEN, -1);
    pancurses::init_pair(3, COLOR_CYAN, -1);
    pancurses::init_pair(4, COLOR_WHITE, COLOR_GREEN);

    let mut panel_song = PanelSong::new(&window, 40, songs)?;

    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::KeyResize) => {
                pancurses::resize_term(0, 0);
            }
            input => panel_song.handle_input(input)?,
        }
    }

    Ok(())
}
