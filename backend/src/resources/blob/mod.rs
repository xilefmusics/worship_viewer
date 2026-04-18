pub use shared::blob::{Blob, CreateBlob, PatchBlob, UpdateBlob};

mod model;
mod repository;
pub mod service;
pub mod storage;
mod surreal_repo;

pub use repository::BlobRepository;
pub use service::{BlobService, BlobServiceHandle};
pub use storage::FsBlobStorage;
pub use surreal_repo::SurrealBlobRepo;

pub mod rest;
