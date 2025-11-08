use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use surrealdb::sql::Thing;

use super::{Role, User};
use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_users(&self) -> Result<Vec<User>, AppError>;
    async fn get_user(&self, id: &str) -> Result<User, AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create_user(&self, user: User) -> Result<User, AppError>;
    async fn delete_user(&self, id: &str) -> Result<User, AppError>;
    async fn get_user_by_email_or_create(&self, email: &str) -> Result<User, AppError>;
}

impl Model for Database {
    async fn get_users(&self) -> Result<Vec<User>, AppError> {
        Ok(self
            .db
            .select("user")
            .await
            .map_err(|err| AppError::database(err))?
            .into_iter()
            .map(UserRecord::into_user)
            .collect())
    }

    async fn get_user(&self, id: &str) -> Result<User, AppError> {
        self.db
            .select(user_resource(id)?)
            .await?
            .map(UserRecord::into_user)
            .ok_or(AppError::NotFound)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self
            .db
            .query("SELECT * FROM user WHERE email = $email LIMIT 1")
            .bind(("email", email.to_lowercase()))
            .await?
            .take::<Option<UserRecord>>(0)?
            .map(UserRecord::into_user))
    }

    async fn create_user(&self, user: User) -> Result<User, AppError> {
        self.db
            .create("user")
            .content(UserRecord::from_user(user))
            .await?
            .map(UserRecord::into_user)
            .ok_or(AppError::database("failed to create user"))
    }

    async fn delete_user(&self, id: &str) -> Result<User, AppError> {
        self.db
            .delete(user_resource(id)?)
            .await?
            .map(UserRecord::into_user)
            .ok_or(AppError::NotFound)
    }

    async fn get_user_by_email_or_create(&self, email: &str) -> Result<User, AppError> {
        if let Some(user) = self.get_user_by_email(email).await? {
            return Ok(user);
        }
        self.create_user(User::new(email.to_lowercase())).await
    }
}

fn user_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "user" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid user id"));
    }

    Ok(("user".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRecord {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    email: String,
    #[serde(default)]
    role: Role,
    #[serde(default)]
    read: Vec<Thing>,
    #[serde(default)]
    write: Vec<Thing>,
    created_at: Datetime,
    #[serde(default)]
    last_login_at: Option<Datetime>,
    #[serde(default)]
    request_count: u64,
}

impl UserRecord {
    pub fn into_user(self) -> User {
        User {
            id: self.id.unwrap().id.to_string(),
            email: self.email,
            role: self.role,
            read: self.read.into_iter().map(|id| id.id.to_string()).collect(),
            write: self.write.into_iter().map(|id| id.id.to_string()).collect(),
            created_at: self.created_at.into(),
            last_login_at: self.last_login_at.map(Into::into),
            request_count: self.request_count,
        }
    }

    pub fn from_user(user: User) -> Self {
        Self {
            id: if user.id.len() > 0 {
                Some(Thing::from(("user".to_owned(), user.id)))
            } else {
                None
            },
            email: user.email,
            role: user.role,
            read: user
                .read
                .into_iter()
                .map(|id| Thing::from(("user".to_owned(), id)))
                .collect(),
            write: user
                .write
                .into_iter()
                .map(|id| Thing::from(("user".to_owned(), id)))
                .collect(),
            created_at: user.created_at.into(),
            last_login_at: user.last_login_at.map(Into::into),
            request_count: user.request_count,
        }
    }
}
