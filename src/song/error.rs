use std::error;
use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub enum Error {
    IO(String),
    SongParse(String),
    Network(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "IO (msg: {})", msg),
            Self::SongParse(msg) => write!(f, "SongParse (msg: {})", msg),
            Self::Network(msg) => write!(f, "Network (msg: {})", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl error::Error for Error {}
