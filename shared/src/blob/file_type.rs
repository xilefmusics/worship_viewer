use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileType {
    #[serde(rename(deserialize = "image/png", serialize = "image/png"))]
    PNG,
    #[serde(rename(deserialize = "image/jpeg", serialize = "image/jpeg"))]
    JPEG,
    #[serde(rename(deserialize = "image/svg", serialize = "image/svg"))]
    SVG,
}

impl FileType {
    pub fn file_ending(&self) -> &'static str {
        match self {
            Self::PNG => ".png",
            Self::JPEG => ".jpeg",
            Self::SVG => ".svg",
        }
    }
}
