pub mod rest;

mod client;
pub use client::{OidcClients, OidcProvider, build_clients};

mod model;
pub(crate) use model::{Model, PendingOidc};
