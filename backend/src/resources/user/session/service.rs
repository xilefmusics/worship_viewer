use std::sync::Arc;

use shared::user::Session;

use crate::database::Database;
use crate::error::AppError;
use crate::resources::user::UserRepository;
use crate::resources::user::surreal_repo::SurrealUserRepo;

use super::repository::SessionRepository;
use super::surreal_repo::SurrealSessionRepo;

/// Application service for session management.
#[derive(Clone)]
pub struct SessionService<S, U> {
    pub repo: S,
    pub user_repo: U,
}

impl<S, U> SessionService<S, U> {
    pub fn new(repo: S, user_repo: U) -> Self {
        Self { repo, user_repo }
    }
}

impl<S: SessionRepository, U: UserRepository> SessionService<S, U> {
    pub async fn get_session(&self, id: &str) -> Result<Session, AppError> {
        self.repo.get_session(id).await
    }

    pub async fn create_session(&self, session: Session) -> Result<Session, AppError> {
        self.repo.create_session(session).await
    }

    pub async fn delete_session(&self, id: &str) -> Result<Session, AppError> {
        self.repo.delete_session(id).await
    }

    pub async fn get_sessions_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, AppError> {
        self.repo.get_sessions_by_user_id(user_id).await
    }

    pub async fn validate_session_and_update_metrics(
        &self,
        id: &str,
    ) -> Result<Option<Session>, AppError> {
        self.repo.validate_session_and_update_metrics(id).await
    }

    /// Look up a user by ID and create a session for them with the given TTL.
    pub async fn create_session_for_user_by_id(
        &self,
        user_id: &str,
        ttl_seconds: i64,
    ) -> Result<Session, AppError> {
        let user = self.user_repo.get_user(user_id).await?;
        self.repo
            .create_session(Session::new(user, ttl_seconds))
            .await
    }
}

/// Production type alias used in HTTP wiring.
pub type SessionServiceHandle = SessionService<SurrealSessionRepo, SurrealUserRepo>;

impl SessionServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        SessionService::new(
            SurrealSessionRepo::new(db.clone()),
            SurrealUserRepo::new(db.clone()),
        )
    }
}
