mod user;
pub use user::{Role, User};

mod model;
pub use model::Model;
use model::UserRecord;

pub mod rest;
pub use rest::CreateUserRequest;

pub mod session;
