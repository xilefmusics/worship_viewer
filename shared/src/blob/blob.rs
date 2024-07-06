use super::FileType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub id: Option<String>,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    pub ocr: String,
}

impl Blob {
    pub fn file_name(&self) -> Option<String> {
        self.id
            .as_ref()
            .map(|id| format!("{}{}", id, self.file_type.file_ending()))
    }
}

#[cfg(feature = "backend")]
impl fancy_surreal::Databasable for Blob {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
