use chordlib::types::Song as SongData;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::song::Song;

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_songs(&self) -> Result<Vec<Song>, AppError>;
    async fn get_song(&self, id: &str) -> Result<Song, AppError>;
    async fn create_song(&self, song: Song) -> Result<Song, AppError>;
    async fn update_song(&self, id: &str, song: Song) -> Result<Song, AppError>;
    async fn delete_song(&self, id: &str) -> Result<Song, AppError>;
}

impl Model for Database {
    async fn get_songs(&self) -> Result<Vec<Song>, AppError> {
        Ok(self
            .db
            .select("song")
            .await?
            .into_iter()
            .map(SongRecord::into_song)
            .collect())
    }

    async fn get_song(&self, id: &str) -> Result<Song, AppError> {
        self.db
            .select(song_resource(id)?)
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::NotFound("song not found".into()))
    }

    async fn create_song(&self, song: Song) -> Result<Song, AppError> {
        self.db
            .create("song")
            .content(SongRecord::from_song(song))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to create song"))
    }

    async fn update_song(&self, id: &str, song: Song) -> Result<Song, AppError> {
        self.db
            .update(song_resource(id)?)
            .content(SongRecord::from_song(song))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::NotFound("song not found".into()))
    }

    async fn delete_song(&self, id: &str) -> Result<Song, AppError> {
        self.db
            .delete(song_resource(id)?)
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::NotFound("song not found".into()))
    }
}

fn song_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "song" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid song id"));
    }

    Ok(("song".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongRecord {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default)]
    not_a_song: bool,
    #[serde(default)]
    blobs: Vec<String>,
    data: SongData,
}

impl SongRecord {
    fn into_song(self) -> Song {
        Song {
            id: self.id.map(|thing| thing.id.to_string()),
            not_a_song: self.not_a_song,
            blobs: self.blobs,
            data: self.data,
        }
    }

    fn from_song(song: Song) -> Self {
        Self {
            id: song
                .id
                .filter(|id| !id.is_empty())
                .map(|id| Thing::from(("song".to_owned(), id))),
            not_a_song: song.not_a_song,
            blobs: song.blobs,
            data: song.data,
        }
    }
}
