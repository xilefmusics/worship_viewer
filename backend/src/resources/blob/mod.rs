pub use shared::blob::{Blob, CreateBlob};

mod model;
mod repository;
pub mod service;
mod surreal_repo;
pub mod storage;

pub use repository::BlobRepository;
pub use service::{BlobService, BlobServiceHandle};
pub use surreal_repo::SurrealBlobRepo;
pub use storage::FsBlobStorage;

pub mod rest;
