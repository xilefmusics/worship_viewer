use std::error;
use std::fmt;
use std::io;

use crate::setlist;
use crate::song;
use crate::tui;

#[derive(Debug, Clone)]
pub enum Error {
    ParseArgs(String),
    ParseSetlist(String),
    FileNotFound(String),
    IO(String),
    WS(String),
    Tui(i32),
    NoSong,
    SongNotFound(String),
    SetlistNotFound(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseArgs(message) => write!(f, "{}", message),
            Self::ParseSetlist(message) => write!(f, "Parsing setlist {}", message),
            Self::FileNotFound(file) => write!(f, "File not found ({})", file),
            Self::Other(message) => write!(f, "{}", message),
            Self::IO(msg) => write!(f, "IO (msg: {})", msg),
            Self::WS(msg) => write!(f, "WS (msg: {})", msg),
            Self::Tui(code) => write!(f, "Tui rendering with code {}", code),
            Self::NoSong => write!(f, "Called render on SongView with no song choosen"),
            Self::SongNotFound(title) => write!(f, "SongNotFound (title: {})", title),
            Self::SetlistNotFound(title) => write!(f, "SongNotFound (title: {})", title),
        }
    }
}

impl From<song::Error> for Error {
    fn from(err: song::Error) -> Self {
        match err {
            song::Error::IO(msg) => Self::IO(msg),
            song::Error::SongNotFound(title) => Self::SongNotFound(title),
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<setlist::Error> for Error {
    fn from(err: setlist::Error) -> Self {
        match err {
            setlist::Error::IO(msg) => Self::IO(msg),
            setlist::Error::SetlistNotFound(title) => Self::SetlistNotFound(title),
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<ws::Error> for Error {
    fn from(err: ws::Error) -> Self {
        Self::WS(err.to_string())
    }
}

impl From<tui::Error> for Error {
    fn from(err: tui::Error) -> Self {
        match err {
            tui::Error::Tui(code) => Self::Tui(code),
        }
    }
}

impl From<i32> for Error {
    fn from(code: i32) -> Self {
        Self::Tui(code)
    }
}

impl error::Error for Error {}
