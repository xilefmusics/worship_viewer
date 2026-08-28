pub use shared::{MediaAsset, MediaAssetKind, MediaAssetStatus};

mod model;
mod repository;
pub mod service;
pub mod storage;
mod surreal_repo;

pub use repository::MediaAssetRepository;
pub use service::{MediaAssetService, MediaAssetServiceHandle, MediaAssetSettings};
pub use storage::FsMediaAssetStorage;
pub use surreal_repo::SurrealMediaAssetRepo;

#[cfg(test)]
pub use model::{CreateFinalAsset, CreateStagingAsset};

pub mod rest;
