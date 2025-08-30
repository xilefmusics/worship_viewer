use fancy_surreal::Record;
use serde::Deserialize;
use shared::song::{Song, SimpleChord};

#[derive(Debug, Deserialize)]
pub struct SongLinkSongs {
    content: SongLinkSongsContent,
}

impl SongLinkSongs {
    pub fn to_songs(self) -> Vec<Song> {
        self.content
            .songs
            .into_iter()
            .map(|item| {
                let mut song = item.id.content();
                if let Some(key) = item.key {
                    song.data.key = Some(key);
                }
                song
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct SongLinkSongsContent {
    songs: Vec<SongLinkSongsContentItems>,
}

#[derive(Debug, Deserialize)]
struct SongLinkSongsContentItems {
    id: Record<Song>,
    key: Option<SimpleChord>,
}
