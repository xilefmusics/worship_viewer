use super::Blob;
use crate::AppError;
use fancy_surreal::Client;
use std::sync::Arc;

pub struct Model;

impl Model {
    pub async fn get(db: Arc<Client<'_>>, owners: Vec<String>) -> Result<Vec<Blob>, AppError> {
        Ok(db.table("blobs").owners(owners).select()?.query().await?)
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Blob, AppError> {
        Ok(db
            .table("blobs")
            .owners(owners)
            .select()?
            .id(id)
            .query_one()
            .await?)
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        blobs: Vec<Blob>,
    ) -> Result<Vec<Blob>, AppError> {
        Ok(db.table("blobs").owners(owners).update(blobs).await?)
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        blobs: Vec<Blob>,
    ) -> Result<Vec<Blob>, AppError> {
        Ok(db.table("blobs").owner(owner).create(blobs).await?)
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        blobs: Vec<Blob>,
    ) -> Result<Vec<Blob>, AppError> {
        Ok(db.table("blobs").owners(owners).delete(blobs).await?)
    }
}
