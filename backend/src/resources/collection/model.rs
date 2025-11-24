use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    collection::{Collection, CreateCollection},
    song::{Link as SongLink, SimpleChord, Song},
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::song::SongRecord;

pub trait Model {
    async fn get_collections(&self, owners: Vec<String>) -> Result<Vec<Collection>, AppError>;
    async fn get_collection(&self, owners: Vec<String>, id: &str) -> Result<Collection, AppError>;
    async fn get_collection_songs(
        &self,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Vec<Song>, AppError>;
    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn update_collection(
        &self,
        owners: Vec<String>,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn delete_collection(
        &self,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Collection, AppError>;
    async fn add_song_to_collection(
        &self,
        owners: Vec<String>,
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError>;
}

impl Model for Database {
    async fn get_collections(&self, owners: Vec<String>) -> Result<Vec<Collection>, AppError> {
        let owners = owners
            .into_iter()
            .map(|owner_id| owner_thing(&owner_id))
            .collect::<Vec<_>>();

        let mut response = self
            .db
            .query("SELECT * FROM collection WHERE owner IN $owners")
            .bind(("owners", owners))
            .await?;

        Ok(response
            .take::<Vec<CollectionRecord>>(0)?
            .into_iter()
            .map(CollectionRecord::into_collection)
            .collect())
    }

    async fn get_collection(&self, owners: Vec<String>, id: &str) -> Result<Collection, AppError> {
        match self.db.select(collection_resource(id)?).await? {
            Some(record) if collection_belongs_to(&record, owners) => Ok(record.into_collection()),
            _ => Err(AppError::NotFound("collection not found".into())),
        }
    }

    async fn get_collection_songs(
        &self,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let resource = collection_resource(id)?;
        let mut response = self
            .db
            .query("SELECT owner, songs FROM collection WHERE id = $id FETCH songs.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<CollectionSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !record.belongs_to(&owners) {
            return Err(AppError::NotFound("collection not found".into()));
        }

        Ok(record.into_songs())
    }

    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        self.db
            .create("collection")
            .content(CollectionRecord::from_payload(
                None,
                Some(owner_thing(owner)),
                collection,
            ))
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to create collection"))
    }

    async fn update_collection(
        &self,
        owners: Vec<String>,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let resource = collection_resource(id)?;
        let existing = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !collection_belongs_to(&existing, owners) {
            return Err(AppError::NotFound("collection not found".into()));
        }

        let record_id = Thing::from(resource.clone());
        let record =
            CollectionRecord::from_payload(Some(record_id), existing.owner.clone(), collection);

        if let Some(updated) = self
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(CollectionRecord::into_collection)
        {
            return Ok(updated);
        }

        self.db
            .create(resource)
            .content(record)
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to upsert collection"))
    }

    async fn delete_collection(
        &self,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let resource = collection_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !collection_belongs_to(&existing, owners) {
                return Err(AppError::NotFound("collection not found".into()));
            }
        } else {
            return Err(AppError::NotFound("collection not found".into()));
        }

        self.db
            .delete(resource)
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn add_song_to_collection(
        &self,
        owners: Vec<String>,
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError> {
        let owners = owners
            .into_iter()
            .map(|owner_id| owner_thing(&owner_id))
            .collect::<Vec<_>>();

        let _ = self
            .db
            .query(
                r#"
            UPDATE type::thing("collection", $id)
            SET songs = array::append(songs, $song)
            WHERE owner IN $owners;
            "#,
            )
            .bind(("id", id.to_owned()))
            .bind(("owners", owners))
            .bind(("song", SongLinkRecord::from(song_link)))
            .await?;

        Ok(())
    }
}

fn collection_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "collection" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid collection id"));
    }

    Ok(("collection".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct CollectionRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    title: String,
    cover: Option<Thing>,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl CollectionRecord {
    fn into_collection(self) -> Collection {
        Collection {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            title: self.title,
            cover: self
                .cover
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, collection: CreateCollection) -> Self {
        Self {
            id,
            owner: owner,
            title: collection.title,
            cover: Some(blob_thing(&collection.cover)),
            songs: collection.songs.into_iter().map(Into::into).collect(),
        }
    }
}

fn blob_thing(blob_id: &str) -> Thing {
    if let Ok(thing) = blob_id.parse::<Thing>() {
        if thing.tb == "blob" {
            return thing;
        }
    }

    Thing::from(("blob".to_owned(), blob_id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SongLinkRecord {
    id: Thing,
    #[serde(default)]
    nr: Option<String>,
    #[serde(default)]
    key: Option<SimpleChord>,
}

impl From<SongLinkRecord> for SongLink {
    fn from(record: SongLinkRecord) -> Self {
        Self {
            id: record.id.id.to_string(),
            nr: record.nr,
            key: record.key,
        }
    }
}

impl From<SongLink> for SongLinkRecord {
    fn from(link: SongLink) -> Self {
        Self {
            id: song_thing(&link.id),
            nr: link.nr,
            key: link.key,
        }
    }
}

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

fn song_thing(song_id: &str) -> Thing {
    if let Ok(thing) = song_id.parse::<Thing>() {
        if thing.tb == "song" {
            return thing;
        }
    }

    Thing::from(("song".to_owned(), song_id.to_owned()))
}

fn collection_belongs_to(record: &CollectionRecord, owners: Vec<String>) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owners.contains(&owner.id.to_string()))
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct CollectionSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl CollectionSongsRecord {
    fn belongs_to(&self, owners: &[String]) -> bool {
        self.owner
            .as_ref()
            .map(|owner| owners.contains(&owner.id.to_string()))
            .unwrap_or(false)
    }

    fn into_songs(self) -> Vec<Song> {
        self.songs
            .into_iter()
            .map(|record| record.into_song())
            .collect()
    }
}

#[derive(Deserialize)]
struct FetchedSongRecord {
    #[serde(rename = "id")]
    song: SongRecord,
}

impl FetchedSongRecord {
    fn into_song(self) -> Song {
        self.song.into_song()
    }
}
