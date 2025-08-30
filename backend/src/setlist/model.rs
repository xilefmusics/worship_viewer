use super::{Setlist, SetlistDatabase};
use crate::AppError;
use fancy_surreal::Client;
use std::sync::Arc;

pub struct Model;

impl Model {
    pub async fn get(db: Arc<Client<'_>>, owners: Vec<String>) -> Result<Vec<Setlist>, AppError> {
        Ok(db
            .table("setlists")
            .owners(owners)
            .select()?
            .query::<SetlistDatabase>()
            .await?
            .into_iter()
            .map(|setlist| setlist.into())
            .collect())
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Setlist, AppError> {
        Ok(db
            .table("setlists")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<SetlistDatabase>()
            .await?
            .into())
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        setlists: Vec<Setlist>,
    ) -> Result<Vec<Setlist>, AppError> {
        Ok(db
            .table("setlists")
            .owners(owners)
            .update::<SetlistDatabase>(setlists.into_iter().map(|setlist| setlist.into()).collect())
            .await?
            .into_iter()
            .map(|setlist| setlist.into())
            .collect())
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        setlists: Vec<Setlist>,
    ) -> Result<Vec<Setlist>, AppError> {
        Ok(db
            .table("setlists")
            .owner(owner)
            .create::<SetlistDatabase>(setlists.into_iter().map(|setlist| setlist.into()).collect())
            .await?
            .into_iter()
            .map(|setlist| setlist.into())
            .collect())
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        setlists: Vec<Setlist>,
    ) -> Result<Vec<Setlist>, AppError> {
        Ok(db
            .table("setlists")
            .owners(owners)
            .delete::<SetlistDatabase>(setlists.into_iter().map(|setlist| setlist.into()).collect())
            .await?
            .into_iter()
            .map(|setlist| setlist.into())
            .collect())
    }
}
