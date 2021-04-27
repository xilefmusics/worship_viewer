use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    IO,
    ParseSetlist(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO => write!(f, "Some IO error occoured"),
            Self::ParseSetlist(msg) => write!(f, "SongParse Error: {}", msg),
        }
    }
}

impl error::Error for Error {}
