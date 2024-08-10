use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn from_dimensions(dimensions: (f64, f64)) -> Self {
        if dimensions.0 > dimensions.1 {
            Self::Landscape
        } else {
            Self::Portrait
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }
}
