use std::error;
use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub enum Error {
    IO(String),
    SongParse(String),
    SongNotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "IO (msg: {})", msg),
            Self::SongParse(msg) => write!(f, "SongParse (msg: {})", msg),
            Self::SongNotFound(title) => write!(f, "SongNotFount (title: {})", title),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl error::Error for Error {}
