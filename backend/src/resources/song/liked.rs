use std::collections::HashSet;

use actix_web::web::Data;
use async_trait::async_trait;

use crate::database::Database;
use crate::error::AppError;

use super::Model as SongDbModel;

/// User-specific liked song ids (backed by the `like` table).
#[async_trait]
pub trait LikedSongIds: Send + Sync {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}

#[async_trait]
impl LikedSongIds for Data<Database> {
    async fn liked_song_ids(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        SongDbModel::get_liked_set(self.get_ref(), user_id).await
    }
}
