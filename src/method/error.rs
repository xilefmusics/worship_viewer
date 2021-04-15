use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    ParseArgs(String),
    FileNotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseArgs(message) => write!(f, "{}", message),
            Self::FileNotFound(file) => write!(f, "File not found ({})", file),
        }

    }
}

impl error::Error for Error {}
