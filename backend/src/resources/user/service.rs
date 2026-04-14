use std::sync::Arc;

use shared::api::ListQuery;
use shared::user::User;

use crate::database::Database;
use crate::error::AppError;
use crate::resources::team::TeamRepository;
use crate::resources::team::model::TeamCreatePayload;
use crate::resources::team::model::user_thing;
use crate::resources::team::surreal_repo::SurrealTeamRepo;

use super::CreateUserRequest;
use super::repository::UserRepository;
use super::surreal_repo::SurrealUserRepo;

/// Application service for user management: creates users with personal teams.
#[derive(Clone)]
pub struct UserService<R, T> {
    pub repo: R,
    pub team_repo: T,
}

impl<R, T> UserService<R, T> {
    pub fn new(repo: R, team_repo: T) -> Self {
        Self { repo, team_repo }
    }
}

impl<R: UserRepository, T: TeamRepository> UserService<R, T> {
    pub async fn get_users(&self, pagination: ListQuery) -> Result<Vec<User>, AppError> {
        self.repo.get_users(pagination).await
    }

    pub async fn get_user(&self, id: &str) -> Result<User, AppError> {
        self.repo.get_user(id).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        self.repo.get_user_by_email(email).await
    }

    /// Create a user and their personal team.
    pub async fn create_user(&self, user: User) -> Result<User, AppError> {
        let created = self.repo.create_user_record(user).await?;
        self.team_repo
            .create_team(TeamCreatePayload {
                name: "Personal".to_owned(),
                owner: Some(user_thing(&created.id)),
                members: vec![],
            })
            .await?;
        Ok(created)
    }

    pub async fn create_user_from_request(
        &self,
        request: CreateUserRequest,
    ) -> Result<User, AppError> {
        self.create_user(request.into_user()).await
    }

    pub async fn get_user_by_email_or_create(&self, email: &str) -> Result<User, AppError> {
        if let Some(user) = self.repo.get_user_by_email(email).await? {
            return Ok(user);
        }
        self.create_user(User::new(email.to_lowercase())).await
    }

    pub async fn delete_user(&self, id: &str) -> Result<User, AppError> {
        self.repo.delete_user(id).await
    }

    pub async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError> {
        self.repo
            .set_default_collection(user_id, collection_id)
            .await
    }
}

/// Production type alias used in HTTP wiring.
pub type UserServiceHandle = UserService<SurrealUserRepo, SurrealTeamRepo>;

impl UserServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        UserService::new(
            SurrealUserRepo::new(db.clone()),
            SurrealTeamRepo::new(db.clone()),
        )
    }
}
