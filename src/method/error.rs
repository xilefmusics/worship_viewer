use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    ParseArgs(String),
    ParseSetlist(String),
    FileNotFound(String),
    IO,
    Tui,
    NoSong,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseArgs(message) => write!(f, "{}", message),
            Self::ParseSetlist(message) => write!(f, "Parsing setlist {}", message),
            Self::FileNotFound(file) => write!(f, "File not found ({})", file),
            Self::Other(message) => write!(f, "{}", message),
            Self::IO => write!(f, "Some IO error occoured"),
            Self::Tui => write!(f, "Some Tui rendering error occoured"),
            Self::NoSong => write!(f, "Called render on SongView with no song choosen"),
        }
    }
}

impl error::Error for Error {}
