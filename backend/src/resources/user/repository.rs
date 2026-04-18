use async_trait::async_trait;

use shared::api::ListQuery;
use shared::user::User;

use crate::error::AppError;

/// Pure user data access — no authorization. All operations work on platform-level identity.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_users(&self, pagination: ListQuery) -> Result<Vec<User>, AppError>;
    /// Count users matching the same optional `q` filter as [`get_users`](Self::get_users) (ignores page).
    async fn count_users(&self, query: ListQuery) -> Result<u64, AppError>;
    async fn get_user(&self, id: &str) -> Result<User, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    /// Insert a user record. Does NOT create a personal team — service layer handles that.
    async fn create_user_record(&self, user: User) -> Result<User, AppError>;
    async fn delete_user(&self, id: &str) -> Result<User, AppError>;
    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError>;
}
