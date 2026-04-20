use std::collections::HashSet;

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::types::{RecordId, SurrealValue};

use crate::database::{Database, record_id_string};
use crate::error::AppError;

/// User-specific liked song ids (backed by the `like` table).
#[async_trait]
pub trait LikedSongIds: Send + Sync {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}

#[derive(Deserialize, SurrealValue)]
struct LikeRow {
    song: RecordId,
}

#[async_trait]
impl LikedSongIds for Arc<Database> {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        let owner = RecordId::new("user", user_id);
        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner")
            .bind(("owner", owner))
            .await?;
        let likes: Vec<LikeRow> = response.take(0)?;
        Ok(likes
            .into_iter()
            .map(|r| record_id_string(&r.song))
            .collect())
    }
}
