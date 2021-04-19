extern crate pancurses;

mod config;
use config::Config;

mod song;
use song::Song;

mod setlist;
use setlist::Setlist;

mod sidebar;
use sidebar::Sidebar;

mod song_view;
use song_view::SongView;

mod panel_song;
use panel_song::PanelSong;

mod tui;
pub use tui::tui;
