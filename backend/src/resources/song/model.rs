use chordlib::types::Song as SongData;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::song::{CreateSong, Song};

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_songs(&self, owners: Vec<String>) -> Result<Vec<Song>, AppError>;
    async fn get_song(&self, owners: Vec<String>, id: &str) -> Result<Song, AppError>;
    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError>;
    async fn update_song(
        &self,
        owners: Vec<String>,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError>;
    async fn delete_song(&self, owners: Vec<String>, id: &str) -> Result<Song, AppError>;
}

impl Model for Database {
    async fn get_songs(&self, owners: Vec<String>) -> Result<Vec<Song>, AppError> {
        let owners = owners
            .into_iter()
            .map(|owner_id| owner_thing(&owner_id))
            .collect::<Vec<_>>();

        let mut response = self
            .db
            .query("SELECT * FROM song WHERE owner IN $owners")
            .bind(("owners", owners))
            .await?;

        Ok(response
            .take::<Vec<SongRecord>>(0)?
            .into_iter()
            .map(SongRecord::into_song)
            .collect())
    }

    async fn get_song(&self, owners: Vec<String>, id: &str) -> Result<Song, AppError> {
        match self.db.select(song_resource(id)?).await? {
            Some(record) if song_belongs_to(&record, owners) => Ok(record.into_song()),
            _ => Err(AppError::NotFound("song not found".into())),
        }
    }

    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError> {
        self.db
            .create("song")
            .content(SongRecord::from_payload(None, Some(owner_thing(owner)), song))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to create song"))
    }

    async fn update_song(
        &self,
        owners: Vec<String>,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let resource = song_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !song_belongs_to(&existing, owners) {
                return Err(AppError::NotFound("song not found".into()));
            }
        }

        let record_id = Thing::from(resource.clone());
        let record = SongRecord::from_payload(Some(record_id), None, song);

        if let Some(updated) = self
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(SongRecord::into_song)
        {
            return Ok(updated);
        }

        self.db
            .create(resource)
            .content(record)
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to upsert song"))
    }

    async fn delete_song(&self, owners: Vec<String>, id: &str) -> Result<Song, AppError> {
        let resource = song_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !song_belongs_to(&existing, owners) {
                return Err(AppError::NotFound("song not found".into()));
            }
        } else {
            return Err(AppError::NotFound("song not found".into()));
        }

        self.db
            .delete(resource)
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SongRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    not_a_song: bool,
    #[serde(default)]
    blobs: Vec<Thing>,
    data: SongData,
}

impl SongRecord {
    fn into_song(self) -> Song {
        Song {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            not_a_song: self.not_a_song,
            blobs: self
                .blobs
                .into_iter()
                .map(|thing| thing.id.to_string())
                .collect(),
            data: self.data,
        }
    }

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, song: CreateSong) -> Self {
        Self {
            id,
            owner: owner,
            not_a_song: song.not_a_song,
            blobs: song
                .blobs
                .into_iter()
                .map(|blob_id| blob_thing(&blob_id))
                .collect(),
            data: song.data,
        }
    }
}

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

fn blob_thing(blob_id: &str) -> Thing {
    if let Ok(thing) = blob_id.parse::<Thing>() {
        if thing.tb == "blob" {
            return thing;
        }
    }

    Thing::from(("blob".to_owned(), blob_id.to_owned()))
}

fn song_belongs_to(record: &SongRecord, owners: Vec<String>) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owners.contains(&owner.id.to_string()))
        .unwrap_or(false)
}
