use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub songs: Vec<String>,
    pub cover: String,
    pub group: String,
    pub tags: Vec<String>,
}
