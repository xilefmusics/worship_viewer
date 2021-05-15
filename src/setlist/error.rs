use std::error;
use std::fmt;
use std::io;

use crate::song;

#[derive(Debug, Clone)]
pub enum Error {
    IO(String),
    ParseSetlist(String),
    Network(String),
    Other(String),
    NoPath,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "IO (msg: {})", msg),
            Self::ParseSetlist(msg) => write!(f, "ParseSetlist (msg: {})", msg),
            Self::Network(msg) => write!(f, "Network (msg: {})", msg),
            Self::Other(msg) => write!(f, "Other (msg: {})", msg),
            Self::NoPath => write!(f, "Try to write setlist with no specified path"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<song::Error> for Error {
    fn from(err: song::Error) -> Self {
        match err {
            song::Error::Network(msg) => Self::Network(msg),
            song::Error::IO(msg) => Self::IO(msg),
            song::Error::SongParse(msg) => Self::IO(msg),
            song::Error::Other(msg) => Self::Other(msg),
            song::Error::NoPath => Self::NoPath,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl error::Error for Error {}
