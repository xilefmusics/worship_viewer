use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    collection::{Collection, CreateCollection},
    song::{Link as SongLink, SimpleChord},
};

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_collections(&self, owner_id: &str) -> Result<Vec<Collection>, AppError>;
    async fn get_collection(&self, owner_id: &str, id: &str) -> Result<Collection, AppError>;
    async fn create_collection(
        &self,
        owner_id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn update_collection(
        &self,
        owner_id: &str,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn delete_collection(&self, owner_id: &str, id: &str) -> Result<Collection, AppError>;
}

impl Model for Database {
    async fn get_collections(&self, owner_id: &str) -> Result<Vec<Collection>, AppError> {
        let mut response = self
            .db
            .query("SELECT * FROM collection WHERE owner = $owner")
            .bind(("owner", owner_thing(owner_id)))
            .await?;

        Ok(response
            .take::<Vec<CollectionRecord>>(0)?
            .into_iter()
            .map(CollectionRecord::into_collection)
            .collect())
    }

    async fn get_collection(&self, owner_id: &str, id: &str) -> Result<Collection, AppError> {
        match self.db.select(collection_resource(id)?).await? {
            Some(record) if collection_belongs_to(&record, owner_id) => {
                Ok(record.into_collection())
            }
            _ => Err(AppError::NotFound("collection not found".into())),
        }
    }

    async fn create_collection(
        &self,
        owner_id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        self.db
            .create("collection")
            .content(CollectionRecord::from_payload(
                None,
                owner_thing(owner_id),
                collection,
            ))
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to create collection"))
    }

    async fn update_collection(
        &self,
        owner_id: &str,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let resource = collection_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !collection_belongs_to(&existing, owner_id) {
                return Err(AppError::NotFound("collection not found".into()));
            }
        }

        let record_id = Thing::from(resource.clone());
        let record =
            CollectionRecord::from_payload(Some(record_id), owner_thing(owner_id), collection);

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

    async fn delete_collection(&self, owner_id: &str, id: &str) -> Result<Collection, AppError> {
        let resource = collection_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !collection_belongs_to(&existing, owner_id) {
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
    cover: String,
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
            cover: self.cover,
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Thing, collection: CreateCollection) -> Self {
        Self {
            id,
            owner: Some(owner),
            title: collection.title,
            cover: collection.cover,
            songs: collection.songs.into_iter().map(Into::into).collect(),
        }
    }
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

fn collection_belongs_to(record: &CollectionRecord, owner_id: &str) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owner.id.to_string() == owner_id)
        .unwrap_or(false)
}
