pub mod invitation;

pub(crate) mod model;
pub mod repository;
pub mod resolver;
pub mod rest;
pub mod service;
pub(crate) mod surreal_repo;

pub use invitation::rest::invitations_accept_scope;
pub use repository::TeamRepository;
pub use resolver::{
    SurrealTeamResolver, TeamResolver, UserPermissions, content_read_team_things,
    content_write_team_things,
};
pub use service::{TeamService, TeamServiceHandle};
pub use surreal_repo::SurrealTeamRepo;
