use std::collections::HashSet;

use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::song::{CreateSong, Song};

use crate::error::AppError;

/// Pure song data access (no user ACL — callers pass pre-resolved team [`Thing`]s).
#[async_trait]
pub trait SongRepository: Send + Sync {
    async fn get_songs(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError>;

    async fn get_song(&self, read_teams: Vec<Thing>, id: &str) -> Result<Song, AppError>;

    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError>;

    async fn update_song(
        &self,
        write_teams: Vec<Thing>,
        actor_user_id: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError>;

    async fn delete_song(&self, write_teams: Vec<Thing>, id: &str) -> Result<Song, AppError>;

    async fn get_song_like(
        &self,
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError>;

    async fn set_song_like(
        &self,
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError>;

    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}
