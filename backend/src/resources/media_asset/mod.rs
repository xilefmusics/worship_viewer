pub use shared::{MediaAsset, MediaAssetKind, MediaAssetStatus, MediaUploadResponse};

mod model;
mod repository;
pub mod service;
pub mod storage;
mod surreal_repo;

pub use repository::MediaAssetRepository;
pub use service::{MediaAssetService, MediaAssetServiceHandle, MediaAssetSettings};
pub use storage::FsMediaAssetStorage;
pub use surreal_repo::SurrealMediaAssetRepo;

pub mod rest;
