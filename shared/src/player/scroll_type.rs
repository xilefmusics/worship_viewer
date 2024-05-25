use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ScrollType {
    #[default]
    OnePage,
    HalfPage,
    TwoPage,
    Book,
    TwoHalfPage,
}

impl ScrollType {
    pub fn next(&self) -> Self {
        match self {
            Self::OnePage => Self::HalfPage,
            Self::HalfPage => Self::TwoPage,
            Self::TwoPage => Self::Book,
            Self::Book => Self::TwoHalfPage,
            Self::TwoHalfPage => Self::OnePage,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnePage => "[1]",
            Self::HalfPage => "[1/2]",
            Self::TwoPage => "[2]",
            Self::Book => "[b]",
            Self::TwoHalfPage => "[2/2]",
        }
    }
}
