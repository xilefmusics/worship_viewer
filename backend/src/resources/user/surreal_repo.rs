use actix_web::web::Data;
use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::user::User;

use crate::database::Database;
use crate::error::AppError;

use super::model::{UserRecord, user_resource};
use super::repository::UserRepository;

#[derive(Clone)]
pub struct SurrealUserRepo {
    db: Data<Database>,
}

impl SurrealUserRepo {
    pub fn new(db: Data<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        self.db.get_ref()
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepo {
    async fn get_users(&self, pagination: ListQuery) -> Result<Vec<User>, AppError> {
        if let Some((offset, limit)) = pagination.to_offset_limit() {
            let mut response = self
                .inner()
                .db
                .query("SELECT * FROM user LIMIT $limit START $start")
                .bind(("limit", limit))
                .bind(("start", offset))
                .await?;

            Ok(response
                .take::<Vec<UserRecord>>(0)?
                .into_iter()
                .map(UserRecord::into_user)
                .collect())
        } else {
            Ok(self
                .inner()
                .db
                .select("user")
                .await?
                .into_iter()
                .map(UserRecord::into_user)
                .collect())
        }
    }

    async fn get_user(&self, id: &str) -> Result<User, AppError> {
        self.inner()
            .db
            .select(user_resource(id)?)
            .await?
            .map(UserRecord::into_user)
            .ok_or(AppError::NotFound("user not found".into()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self
            .inner()
            .db
            .query("SELECT * FROM user WHERE email = $email LIMIT 1")
            .bind(("email", email.to_lowercase()))
            .await?
            .take::<Option<UserRecord>>(0)?
            .map(UserRecord::into_user))
    }

    async fn create_user_record(&self, user: User) -> Result<User, AppError> {
        self.inner()
            .db
            .create("user")
            .content(UserRecord::from_user(user))
            .await?
            .map(UserRecord::into_user)
            .ok_or_else(|| AppError::database("failed to create user"))
    }

    async fn delete_user(&self, id: &str) -> Result<User, AppError> {
        self.inner()
            .db
            .delete(user_resource(id)?)
            .await?
            .map(UserRecord::into_user)
            .ok_or(AppError::NotFound("user not found".into()))
    }

    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError> {
        let _ = self
            .inner()
            .db
            .query("UPDATE $user SET default_collection = $collection")
            .bind(("user", Thing::from(("user".to_owned(), user_id.to_owned()))))
            .bind((
                "collection",
                Thing::from(("collection".to_owned(), collection_id.to_owned())),
            ))
            .await?;
        Ok(())
    }
}
