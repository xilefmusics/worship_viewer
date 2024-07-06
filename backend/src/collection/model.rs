use super::{Collection, CollectionDatabase};
use crate::AppError;
use fancy_surreal::Client;
use std::sync::Arc;

pub struct Model;

impl Model {
    pub async fn get(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .query::<CollectionDatabase>()
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Collection, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<CollectionDatabase>()
            .await?
            .into())
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .update::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owner(owner)
            .create::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .delete::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn get_song_link_numbers(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Vec<String>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .id(id)
            .field("content.songs.nr")
            .wrapper_js_map_unpack("element.content.songs.nr")
            .query_direct::<String>()
            .await?)
    }
}
