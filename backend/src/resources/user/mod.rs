pub use shared::user::{CreateUserRequest, Role, User};

mod model;
pub use model::{Model, UserRecord};

pub mod rest;

pub mod session;
