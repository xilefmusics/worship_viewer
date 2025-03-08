use fancy_surreal::Record;
use serde::Deserialize;
use shared::song::Song;

#[derive(Debug, Deserialize)]
pub struct CollectionSongs {
    content: CollectionContent,
}

impl CollectionSongs {
    pub fn to_songs(self) -> Vec<Song> {
        self.content
            .songs
            .id
            .into_iter()
            .map(|record| record.content())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct CollectionContent {
    songs: CollectionContentSongs,
}

#[derive(Debug, Deserialize)]
struct CollectionContentSongs {
    id: Vec<Record<Song>>,
}
