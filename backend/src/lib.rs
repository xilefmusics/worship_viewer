#![allow(async_fn_in_trait)]

pub mod auth;
pub mod database;
pub mod docs;
pub mod error;
pub mod http_cache;
pub mod frontend;
pub mod http_cache;
pub mod mail;
pub mod request_id;
pub mod resources;
pub mod settings;

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod http_tests;
