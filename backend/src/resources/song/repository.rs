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
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError>;

    async fn get_song(&self, read_teams: &[Thing], id: &str) -> Result<Song, AppError>;

    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError>;

    /// Update an existing song, or create it if it doesn't yet exist
    /// (upsert semantics). This supports import/sync workflows where the
    /// caller specifies the song ID.
    ///
    /// - If the song exists and `write_teams` includes its owner, updates
    ///   and returns the song.
    /// - If the song exists but the caller lacks write access, returns
    ///   [`AppError::NotFound`].
    /// - If the song does not exist and the actor's personal team is in
    ///   `write_teams`, creates it with the given `id` under that team.
    /// - Otherwise returns [`AppError::NotFound`].
    async fn update_song(
        &self,
        write_teams: &[Thing],
        actor_user_id: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError>;

    async fn delete_song(&self, write_teams: &[Thing], id: &str) -> Result<Song, AppError>;

    /// Count all songs visible to `read_teams`, optionally filtered by `q`.
    async fn count_songs(&self, read_teams: &[Thing], q: Option<&str>) -> Result<u64, AppError>;

    async fn get_song_like(
        &self,
        read_teams: &[Thing],
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError>;

    async fn set_song_like(
        &self,
        read_teams: &[Thing],
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError>;

    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}
