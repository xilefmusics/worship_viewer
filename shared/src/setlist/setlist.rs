use crate::song::Link as SongLink;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Setlist {
    pub id: Option<String>,
    pub title: String,
    pub songs: Vec<SongLink>,
}