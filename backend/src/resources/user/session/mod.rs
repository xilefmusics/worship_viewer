pub use shared::user::Session;

pub(crate) mod model;

pub mod repository;
pub use repository::SessionRepository;

pub(crate) mod surreal_repo;
pub use surreal_repo::SurrealSessionRepo;

pub mod service;
pub use service::{SessionService, SessionServiceHandle};

pub mod rest;
