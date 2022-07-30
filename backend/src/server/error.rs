use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for Error {}
