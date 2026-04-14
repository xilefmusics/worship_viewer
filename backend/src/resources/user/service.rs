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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use surrealdb::sql::Thing;

    use shared::api::ListQuery;
    use shared::team::Team;
    use shared::user::{Role, User};

    use crate::database::record_id_string;
    use crate::error::AppError;
    use crate::resources::team::model::{
        DbTeamMember, TeamCreatePayload, TeamFetched,
    };
    use crate::resources::team::repository::TeamRepository;
    use crate::resources::user::repository::UserRepository;

    use super::UserService;

    // ── Test data helpers ─────────────────────────────────────────────────────

    fn make_user(id: &str, email: &str) -> User {
        let mut u = User::new(email);
        u.id = id.to_owned();
        u
    }

    // ── MockUserRepo ──────────────────────────────────────────────────────────

    struct MockUserRepo {
        user_by_email: Option<User>,
        create_fails_with: Option<AppError>,
    }

    impl MockUserRepo {
        fn empty() -> Self {
            Self {
                user_by_email: None,
                create_fails_with: None,
            }
        }

        fn with_existing_user(user: User) -> Self {
            Self {
                user_by_email: Some(user),
                create_fails_with: None,
            }
        }

        fn failing_create(err: AppError) -> Self {
            Self {
                user_by_email: None,
                create_fails_with: Some(err),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn get_users(&self, _pagination: ListQuery) -> Result<Vec<User>, AppError> {
            unreachable!("not used in these tests")
        }

        async fn get_user(&self, _id: &str) -> Result<User, AppError> {
            unreachable!("not used in these tests")
        }

        async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>, AppError> {
            Ok(self.user_by_email.clone())
        }

        async fn create_user_record(&self, user: User) -> Result<User, AppError> {
            if let Some(ref err) = self.create_fails_with {
                return Err(AppError::conflict(err.to_string()));
            }
            let mut created = user;
            if created.id.is_empty() {
                created.id = "new-user-id".to_owned();
            }
            Ok(created)
        }

        async fn delete_user(&self, _id: &str) -> Result<User, AppError> {
            unreachable!("not used in these tests")
        }

        async fn set_default_collection(
            &self,
            _user_id: &str,
            _collection_id: &str,
        ) -> Result<(), AppError> {
            unreachable!("not used in these tests")
        }
    }

    // ── MockTeamRepo ──────────────────────────────────────────────────────────

    struct MockTeamRepo {
        captured_owner: Arc<Mutex<Option<Thing>>>,
    }

    impl MockTeamRepo {
        fn new() -> Self {
            Self {
                captured_owner: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl TeamRepository for MockTeamRepo {
        async fn create_team(&self, payload: TeamCreatePayload) -> Result<String, AppError> {
            *self.captured_owner.lock().unwrap() = payload.owner;
            Ok("personal-team-id".to_owned())
        }

        async fn fetch_all_teams(&self) -> Result<Vec<TeamFetched>, AppError> {
            unreachable!("not used in user tests")
        }

        async fn fetch_teams_for_user(
            &self,
            _user_id: &str,
            _is_admin: bool,
        ) -> Result<Vec<TeamFetched>, AppError> {
            unreachable!("not used in user tests")
        }

        async fn fetch_team(&self, _id: &str) -> Result<Option<TeamFetched>, AppError> {
            unreachable!("not used in user tests")
        }

        async fn update_team_name(
            &self,
            _resource: (String, String),
            _name: &str,
        ) -> Result<(), AppError> {
            unreachable!("not used in user tests")
        }

        async fn update_team_members(
            &self,
            _resource: (String, String),
            _members: Vec<DbTeamMember>,
        ) -> Result<(), AppError> {
            unreachable!("not used in user tests")
        }

        async fn delete_team_record(
            &self,
            _resource: (String, String),
        ) -> Result<(), AppError> {
            unreachable!("not used in user tests")
        }

        async fn reassign_content(
            &self,
            _from: Thing,
            _to: Thing,
        ) -> Result<(), AppError> {
            unreachable!("not used in user tests")
        }

        async fn load_team_display(&self, _id: &str) -> Result<Team, AppError> {
            unreachable!("not used in user tests")
        }
    }

    // ── Slice 2D: user creation and email ─────────────────────────────────────

    /// BLC-USER-003: creating a user also creates a personal team with that user as owner.
    #[tokio::test]
    async fn blc_user_003_create_user_creates_personal_team() {
        let mock_team = MockTeamRepo::new();
        let captured_owner = mock_team.captured_owner.clone();
        let svc = UserService::new(MockUserRepo::empty(), mock_team);
        let user = User::new("user@example.com");
        let result = svc.create_user(user).await.unwrap();
        let owner = captured_owner.lock().unwrap();
        let owner_thing = owner.as_ref().expect("create_team must be called with an owner");
        assert_eq!(owner_thing.tb, "user");
        assert_eq!(record_id_string(owner_thing), result.id);
    }

    /// BLC-USER-002: a newly created user always has the default role.
    #[tokio::test]
    async fn blc_user_002_new_user_has_default_role() {
        let svc = UserService::new(MockUserRepo::empty(), MockTeamRepo::new());
        let user = User::new("user@example.com");
        let result = svc.create_user(user).await.unwrap();
        assert_eq!(result.role, Role::Default);
    }

    /// BLC-USER-001: get_user_by_email_or_create creates a user when the email is new.
    #[tokio::test]
    async fn blc_user_001_get_by_email_or_create_new_email() {
        let svc = UserService::new(MockUserRepo::empty(), MockTeamRepo::new());
        let result = svc.get_user_by_email_or_create("new@example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "new@example.com");
    }

    /// BLC-USER-001: get_user_by_email_or_create returns the existing user without creating a new one.
    #[tokio::test]
    async fn blc_user_001_get_by_email_or_create_existing_email() {
        let existing = make_user("existing-id", "exists@example.com");
        let svc = UserService::new(MockUserRepo::with_existing_user(existing.clone()), MockTeamRepo::new());
        let result = svc.get_user_by_email_or_create("exists@example.com").await.unwrap();
        assert_eq!(result.id, existing.id);
    }

    /// BLC-USER-008: duplicate email during create propagates the conflict error.
    #[tokio::test]
    async fn blc_user_008_create_duplicate_email_conflict() {
        let svc = UserService::new(
            MockUserRepo::failing_create(AppError::conflict("duplicate email")),
            MockTeamRepo::new(),
        );
        let user = User::new("dup@example.com");
        let result = svc.create_user(user).await;
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }
}
