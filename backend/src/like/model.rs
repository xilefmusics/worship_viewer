use super::{Like, LikeDatabase};
use crate::AppError;
use fancy_surreal::Client;
use fancy_surreal::RecordId;
use std::sync::Arc;

pub struct Model {}

impl Model {
    pub async fn get(db: Arc<Client<'_>>, owners: Vec<String>) -> Result<Vec<Like>, AppError> {
        Ok(db
            .table("likes")
            .owners(owners)
            .select()?
            .query::<LikeDatabase>()
            .await?
            .into_iter()
            .map(|like| like.into())
            .collect())
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Like, AppError> {
        Ok(db
            .table("likes")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<LikeDatabase>()
            .await?
            .into())
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        likes: Vec<Like>,
    ) -> Result<Vec<Like>, AppError> {
        Ok(db
            .table("likes")
            .owners(owners)
            .update::<LikeDatabase>(likes.into_iter().map(|like| like.into()).collect())
            .await?
            .into_iter()
            .map(|like| like.into())
            .collect())
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        likes: Vec<Like>,
    ) -> Result<Vec<Like>, AppError> {
        Ok(db
            .table("likes")
            .owner(owner)
            .create::<LikeDatabase>(likes.into_iter().map(|like| like.into()).collect())
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        likes: Vec<Like>,
    ) -> Result<Vec<Like>, AppError> {
        Ok(db
            .table("likes")
            .owners(owners)
            .delete::<LikeDatabase>(likes.into_iter().map(|like| like.into()).collect())
            .await?
            .into_iter()
            .map(|like| like.into())
            .collect())
    }

    pub async fn toggle(db: Arc<Client<'_>>, owner: &str, song: &str) -> Result<bool, AppError> {
        let db = db.table("likes").owner(owner);

        let likes = db
            .clone()
            .select()?
            .condition(&format!("content.song = songs:{}", song))
            .query::<LikeDatabase>()
            .await?;

        if likes.len() > 0 {
            db.delete(likes).await?;
            Ok(false)
        } else {
            db.create_one::<LikeDatabase>(
                Like {
                    id: None,
                    song: song.into(),
                }
                .into(),
            )
            .await?;
            Ok(true)
        }
    }

    pub async fn filter_liked(
        db: Arc<Client<'_>>,
        song_ids: &[String],
        owners: Vec<String>,
    ) -> Result<Vec<String>, AppError> {
        Ok(db
            .table("likes")
            .owners(owners)
            .select()?
            .condition(&format!(
                "content.song in [{}]",
                song_ids
                    .iter()
                    .map(|id| format!("songs:{}", id))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .field("content.song")
            .wrapper_js_map("element.content.song")
            .query_direct::<RecordId>()
            .await?
            .into_iter()
            .map(|id| id.key().to_string())
            .collect::<Vec<String>>())
    }
}
