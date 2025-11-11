use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{collection::Collection, song::Link as SongLink};

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_collections(&self) -> Result<Vec<Collection>, AppError>;
    async fn get_collection(&self, id: &str) -> Result<Collection, AppError>;
    async fn create_collection(&self, collection: Collection) -> Result<Collection, AppError>;
    async fn update_collection(
        &self,
        id: &str,
        collection: Collection,
    ) -> Result<Collection, AppError>;
    async fn delete_collection(&self, id: &str) -> Result<Collection, AppError>;
}

impl Model for Database {
    async fn get_collections(&self) -> Result<Vec<Collection>, AppError> {
        Ok(self
            .db
            .select("collection")
            .await?
            .into_iter()
            .map(CollectionRecord::into_collection)
            .collect())
    }

    async fn get_collection(&self, id: &str) -> Result<Collection, AppError> {
        self.db
            .select(collection_resource(id)?)
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn create_collection(&self, collection: Collection) -> Result<Collection, AppError> {
        self.db
            .create("collection")
            .content(CollectionRecord::from_collection(collection))
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to create collection"))
    }

    async fn update_collection(
        &self,
        id: &str,
        collection: Collection,
    ) -> Result<Collection, AppError> {
        self.db
            .update(collection_resource(id)?)
            .content(CollectionRecord::from_collection(collection))
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn delete_collection(&self, id: &str) -> Result<Collection, AppError> {
        self.db
            .delete(collection_resource(id)?)
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CollectionRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    title: String,
    cover: String,
    #[serde(default)]
    songs: Vec<SongLink>,
}

impl CollectionRecord {
    fn into_collection(self) -> Collection {
        Collection {
            id: self.id.map(|thing| thing.id.to_string()),
            title: self.title,
            cover: self.cover,
            songs: self.songs,
        }
    }

    fn from_collection(collection: Collection) -> Self {
        Self {
            id: collection
                .id
                .filter(|id| !id.is_empty())
                .map(|id| Thing::from(("collection".to_owned(), id))),
            title: collection.title,
            cover: collection.cover,
            songs: collection.songs,
        }
    }
}
