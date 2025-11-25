use std::collections::HashSet;

use chordlib::types::Song as SongData;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

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
        owner: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError>;
    async fn delete_song(&self, owners: Vec<String>, id: &str) -> Result<Song, AppError>;
    async fn get_song_like(
        &self,
        owners: Vec<String>,
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError>;
    async fn set_song_like(
        &self,
        owners: Vec<String>,
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError>;
    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}

impl Model for Database {
    async fn get_songs(&self, owners: Vec<String>) -> Result<Vec<Song>, AppError> {
        let owners = owners
            .into_iter()
            .map(|owner| owner_thing(&owner))
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
            .content(SongRecord::from_payload(
                None,
                Some(owner_thing(owner)),
                song,
            ))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to create song"))
    }

    async fn update_song(
        &self,
        owners: Vec<String>,
        owner: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let resource = song_resource(id)?;
        let owner = if let Some(existing) = self.db.select(resource.clone()).await? {
            if !song_belongs_to(&existing, owners) {
                return Err(AppError::NotFound("song not found".into()));
            }
            existing.owner
        } else {
            // Check write permission before creating new song
            if !owners.contains(&owner.to_string()) {
                return Err(AppError::NotFound("song not found".into()));
            }
            Some(owner_thing(owner))
        };

        let record_id = Thing::from(resource.clone());
        let record = SongRecord::from_payload(Some(record_id), owner.clone(), song);

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

    async fn get_song_like(
        &self,
        owners: Vec<String>,
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError> {
        let resource = song_resource(id)?;
        let existing = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !song_belongs_to(&existing, owners) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner AND song = $song LIMIT 1")
            .bind(("owner", owner))
            .bind(("song", song))
            .await?;

        let likes: Vec<LikeRecord> = response.take(0)?;
        Ok(!likes.is_empty())
    }

    async fn set_song_like(
        &self,
        owners: Vec<String>,
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError> {
        let resource = song_resource(id)?;
        let existing = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !song_belongs_to(&existing, owners) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner AND song = $song LIMIT 1")
            .bind(("owner", owner.clone()))
            .bind(("song", song.clone()))
            .await?;

        let mut likes: Vec<LikeRecord> = response.take(0)?;
        let existing_like = likes.pop();

        if liked {
            if existing_like.is_none() {
                let _: Option<LikeRecord> = self
                    .db
                    .create("like")
                    .content(LikeRecord::new(owner, song))
                    .await?;
            }
            Ok(true)
        } else if let Some(record) = existing_like.and_then(|like| like.id) {
            let resource = (record.tb.clone(), record.id.to_string());
            let _: Option<LikeRecord> = self.db.delete(resource).await?;
            Ok(false)
        } else {
            Ok(false)
        }
    }
    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner")
            .bind(("owner", owner_thing(user_id)))
            .await?;

        let likes: Vec<LikeRecord> = response.take(0)?;
        Ok(likes
            .into_iter()
            .map(|like| like.song.id.to_string())
            .collect())
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
    pub fn into_song(self) -> Song {
        Song {
            id: self.id.map(id_from_thing).unwrap_or_default(),
            owner: self.owner.map(id_from_thing).unwrap_or_default(),
            not_a_song: self.not_a_song,
            blobs: self.blobs.into_iter().map(id_from_thing).collect(),
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

fn id_from_thing(thing: Thing) -> String {
    id_to_plain_string(thing.id)
}

fn id_to_plain_string(id: Id) -> String {
    match id {
        Id::String(value) => value,
        Id::Number(number) => format!("{number}"),
        Id::Uuid(uuid) => uuid.to_string(),
        _ => id.to_string(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LikeRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    owner: Thing,
    song: Thing,
}

impl LikeRecord {
    fn new(owner: Thing, song: Thing) -> Self {
        Self {
            id: None,
            owner,
            song,
        }
    }
}
