use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    IO,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO => write!(f, "Some IO error occoured"),
        }
    }
}

impl error::Error for Error {}