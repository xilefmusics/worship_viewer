use pancurses::{endwin, initscr};
use pancurses::{Input, Window};
use pancurses::{COLOR_CYAN, COLOR_GREEN, COLOR_RED, COLOR_WHITE};

use std::env;
use std::sync::Arc;

use crate::setlist::SetlistPool;
use crate::song::SongPool;

use super::super::Error;

use super::{Config, PanelSetlist, PanelSong};

pub fn tui(args: env::Args) -> Result<(), Error> {
    let window = initscr();
    let result = tui_inner(args, &window);
    endwin();
    result
}

fn tui_inner(args: env::Args, window: &Window) -> Result<(), Error> {
    let config = Config::new(args)?;

    let (song_pool, setlist_pool) = match &config.url {
        Some(url) => (
            Arc::new(SongPool::new_remote(url.clone())),
            Arc::new(SetlistPool::new_remote(url.clone())?),
        ),
        None => {
            let song_pool = Arc::new(SongPool::new_local(config.root_path.expect("Not remote"))?);
            let setlist_pool = Arc::new(SetlistPool::new_local(
                config.setlist_path.expect("Not remote"),
                Arc::clone(&song_pool),
            )?);
            (song_pool, setlist_pool)
        }
    };

    let mut curr_pannel = 1;

    pancurses::noecho();
    pancurses::curs_set(0);
    pancurses::start_color();
    pancurses::use_default_colors();
    pancurses::init_pair(0, -1, -1);
    pancurses::init_pair(1, COLOR_RED, -1);
    pancurses::init_pair(2, COLOR_GREEN, -1);
    pancurses::init_pair(3, COLOR_CYAN, -1);
    pancurses::init_pair(4, COLOR_WHITE, COLOR_GREEN);

    let mut panel_setlist = PanelSetlist::new(
        window.get_max_y(),
        window.get_max_x(),
        0,
        0,
        &window,
        Arc::clone(&song_pool),
        Arc::clone(&setlist_pool),
    )?;

    let mut panel_song = PanelSong::new(&window, 40, song_pool, setlist_pool)?;

    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::KeyResize) => {
                pancurses::resize_term(0, 0);
            }
            Some(Input::Character('1')) => {
                curr_pannel = 1;
                panel_song.render()?;
            }
            Some(Input::Character('2')) => {
                curr_pannel = 2;
                panel_setlist.render();
            }
            input => {
                if curr_pannel == 1 {
                    panel_song.handle_input(input)
                } else if curr_pannel == 2 {
                    panel_setlist.handle_input(input)
                } else {
                    Ok(())
                }?
            }
        }
    }

    Ok(())
}
