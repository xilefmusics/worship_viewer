extern crate pancurses;

mod song;
use song::Song;

mod config;
use config::Config;

mod sidebar;
use sidebar::Sidebar;

mod song_view;
use song_view::SongView;

mod tui;
pub use tui::tui;
