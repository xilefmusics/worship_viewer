use async_trait::async_trait;
use surrealdb::types::RecordId;

use shared::api::ListQuery;
use shared::setlist::{CreateSetlist, Setlist};
use shared::song::LinkOwned as SongLinkOwned;

use crate::error::AppError;

/// Pure setlist data access (no user ACL — callers pass pre-resolved team [`RecordId`]s).
#[async_trait]
pub trait SetlistRepository: Send + Sync {
    async fn get_setlists(
        &self,
        read_teams: &[RecordId],
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError>;

    /// Count all setlists visible to `read_teams`, optionally filtered by `q`.
    async fn count_setlists(
        &self,
        read_teams: &[RecordId],
        q: Option<&str>,
    ) -> Result<u64, AppError>;

    async fn get_setlist(&self, read_teams: &[RecordId], id: &str) -> Result<Setlist, AppError>;

    async fn get_setlist_songs(
        &self,
        read_teams: &[RecordId],
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError>;

    async fn create_setlist(
        &self,
        owner: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;

    async fn update_setlist(
        &self,
        write_teams: &[RecordId],
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;

    async fn delete_setlist(&self, write_teams: &[RecordId], id: &str)
    -> Result<Setlist, AppError>;
}
