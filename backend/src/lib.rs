#![allow(async_fn_in_trait)]

pub mod accept;
pub mod auth;
pub mod database;
pub mod docs;
pub mod error;
pub mod expand;
pub mod frontend;
pub mod governor_audit;
pub mod governor_peer;
pub mod http_audit;
pub mod http_cache;
pub mod mail;
pub mod observability;
pub mod request_id;
pub mod request_link;
pub mod resources;
pub mod settings;

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod http_tests;

#[cfg(test)]
mod audit_events_tests;
