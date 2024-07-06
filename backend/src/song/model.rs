use super::{Filter, Song};
use crate::AppError;
use fancy_surreal::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Model;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SongCollectionWrapper {
    pub id: Option<String>,
    pub songs: Vec<String>,
}

impl fancy_surreal::Databasable for SongCollectionWrapper {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}

impl Model {
    pub async fn get(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        filter: &Filter<'_>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(if let Some(id) = &filter.get_id() {
            db.table("songs")
                .owners(owners)
                .select()?
                .id(id)
                .query()
                .await?
        } else if let Some(collection) = &filter.get_collection() {
            db.table("collections")
                .owners(owners.clone())
                .select()?
                .id(collection)
                .field("content.songs.id")
                .fetch("content.songs.id")
                .wrapper_js_map_unpack("element.content.songs.id")
                .query::<Song>()
                .await?
        } else {
            db.table("songs").owners(owners).select()?.query().await?
        })
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Song, AppError> {
        Ok(db
            .table("songs")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<Song>()
            .await?)
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(db.table("songs").owners(owners).update(songs).await?)
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(db.table("songs").owner(owner).create(songs).await?)
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(db.table("songs").owners(owners).delete(songs).await?)
    }
}
