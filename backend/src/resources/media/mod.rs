pub use shared::media::{
    CreateMedia, CreateMediaContent, DeclaredMediaKind, DuplicateMedia, LivestreamType, Media,
    MediaContent, MediaDeckPage, MediaPendingRevision, MediaProcessingError, MediaStatus,
    UpdateMedia,
};

mod model;
mod repository;
pub mod rest;
pub mod service;
mod surreal_repo;

pub mod av_processor;
pub mod processing;

pub use repository::MediaRepository;
pub use service::{MediaService, MediaServiceHandle};
pub use surreal_repo::SurrealMediaRepo;
