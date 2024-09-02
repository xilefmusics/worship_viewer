use chordlib::types::Song as SongData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Song {
    pub id: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    pub data: SongData,
}

#[cfg(feature = "backend")]
impl fancy_surreal::Databasable for Song {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
