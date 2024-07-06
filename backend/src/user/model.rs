use super::User;
use crate::AppError;
use fancy_surreal::Client;
use std::sync::Arc;

pub struct Model;

impl Model {
    pub async fn get(db: Arc<Client<'_>>, owners: Vec<String>) -> Result<Vec<User>, AppError> {
        Ok(db.table("users").owners(owners).select()?.query().await?)
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<User, AppError> {
        Ok(db
            .table("users")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<User>()
            .await?)
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        users: Vec<User>,
    ) -> Result<Vec<User>, AppError> {
        Ok(db.table("users").owners(owners).update(users).await?)
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        users: Vec<User>,
    ) -> Result<Vec<User>, AppError> {
        Ok(db.table("users").owners(owners).delete(users).await?)
    }

    pub async fn get_or_create(db: Arc<Client<'_>>, id: &str) -> Result<User, AppError> {
        let user = Self::get_one(db.clone(), vec!["admin".to_string()], id).await;
        if user.is_ok() {
            return user;
        }

        Self::create(
            db,
            vec![User {
                id: Some(id.to_string()),
                read: vec![id.to_string()],
                write: vec![id.to_string()],
            }],
        )
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::Other(
            "This vector should never be empty".to_string(),
        ))
    }

    pub async fn create(db: Arc<Client<'_>>, users: Vec<User>) -> Result<Vec<User>, AppError> {
        Ok(db.table("users").owner("admin").create(users).await?)
    }

    pub async fn admin_write_or_unauthorized(
        db: Arc<Client<'_>>,
        id: &str,
    ) -> Result<(), AppError> {
        if Self::get_or_create(db, id)
            .await?
            .write
            .contains(&"admin".to_string())
        {
            Ok(())
        } else {
            Err(AppError::Unauthorized("no admin write rights".to_string()))
        }
    }
}
