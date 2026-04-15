use async_trait::async_trait;

use shared::user::Session;

use crate::error::AppError;

/// Pure session data access — no authorization.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn get_session(&self, id: &str) -> Result<Session, AppError>;
    async fn get_session_for_user(&self, id: &str, user_id: &str) -> Result<Session, AppError>;
    async fn create_session(&self, session: Session) -> Result<Session, AppError>;
    async fn delete_session(&self, id: &str) -> Result<Session, AppError>;
    async fn get_sessions_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, AppError>;
    /// Atomically validates the session (deleting if expired) and updates user metrics.
    /// Returns `None` when the session does not exist or was just deleted as expired.
    async fn validate_session_and_update_metrics(
        &self,
        id: &str,
    ) -> Result<Option<Session>, AppError>;
}
