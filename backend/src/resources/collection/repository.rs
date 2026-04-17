use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::collection::{Collection, CreateCollection};
use shared::song::{Link as SongLink, LinkOwned as SongLinkOwned};

use crate::error::AppError;

/// Pure collection data access (no user ACL — callers pass pre-resolved team [`Thing`]s).
#[async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn get_collections(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError>;

    /// Count all collections visible to `read_teams`, optionally filtered by `q`.
    async fn count_collections(&self, read_teams: &[Thing], q: Option<&str>) -> Result<u64, AppError>;

    async fn get_collection(&self, read_teams: &[Thing], id: &str) -> Result<Collection, AppError>;

    async fn get_collection_songs(
        &self,
        read_teams: &[Thing],
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError>;

    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;

    async fn update_collection(
        &self,
        write_teams: &[Thing],
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;

    async fn delete_collection(
        &self,
        write_teams: &[Thing],
        id: &str,
    ) -> Result<Collection, AppError>;

    async fn add_song_to_collection(
        &self,
        write_teams: &[Thing],
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError>;
}
