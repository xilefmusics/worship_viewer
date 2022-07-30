use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    Tui(i32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Tui(code) => write!(f, "Tui rendering with code {}", code),
        }
    }
}

impl From<i32> for Error {
    fn from(code: i32) -> Self {
        Self::Tui(code)
    }
}

impl error::Error for Error {}
