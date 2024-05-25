use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum SelectType {
    #[default]
    Item,
    Song,
}

impl SelectType {
    pub fn next(&self) -> Self {
        match self {
            Self::Item => Self::Song,
            Self::Song => Self::Item,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Item => "[pg]",
            Self::Song => "[nr]",
        }
    }
}
