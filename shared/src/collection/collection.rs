use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
    pub id: Option<String>,
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLink>,
}
