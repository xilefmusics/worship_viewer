mod config;
use config::Config;

mod song_view;
use song_view::SongView;

mod panel_song;
use panel_song::PanelSong;

mod tui;
pub use tui::tui;
