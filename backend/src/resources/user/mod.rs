pub use shared::user::{CreateUserRequest, Role, User};

pub(crate) mod model;
pub use model::UserRecord;

pub mod repository;
pub use repository::UserRepository;

pub(crate) mod surreal_repo;
pub use surreal_repo::SurrealUserRepo;

pub mod service;
pub use service::{UserService, UserServiceHandle};

pub mod rest;

pub mod session;
