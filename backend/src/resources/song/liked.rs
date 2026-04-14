use std::collections::HashSet;

use actix_web::web::Data;
use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::sql::Thing;

use crate::database::Database;
use crate::error::AppError;

/// User-specific liked song ids (backed by the `like` table).
#[async_trait]
pub trait LikedSongIds: Send + Sync {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}

#[derive(Deserialize)]
struct LikeRow {
    song: Thing,
}

#[async_trait]
impl LikedSongIds for Data<Database> {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        let owner = Thing::from(("user".to_owned(), user_id.to_owned()));
        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner")
            .bind(("owner", owner))
            .await?;
        let likes: Vec<LikeRow> = response.take(0)?;
        Ok(likes.into_iter().map(|r| r.song.id.to_string()).collect())
    }
}
