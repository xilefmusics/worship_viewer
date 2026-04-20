//! Public deployment metadata (`GET /api/v1/about`).

use actix_web::{HttpResponse, get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::observability;

const SERVICE_NAME: &str = "worshipviewer-backend";

/// JSON body for [`get_about`].
#[derive(Serialize, ToSchema)]
pub struct AboutResponse {
    /// Identifies this server binary.
    pub service: &'static str,
    /// Semver from `Cargo.toml` at compile time (`CARGO_PKG_VERSION`).
    pub version: &'static str,
    /// Git revision if `GIT_COMMIT_SHA` was set during `cargo build`.
    pub git_commit: Option<&'static str>,
    /// `true` when [`observability::is_production`] is satisfied.
    pub production: bool,
}

/// Public metadata: which build is running (no authentication).
#[utoipa::path(
    get,
    path = "/api/v1/about",
    responses(
        (status = 200, description = "Deployed backend metadata", body = AboutResponse),
    ),
    tag = "About",
)]
#[get("/about")]
pub async fn get_about() -> HttpResponse {
    HttpResponse::Ok().json(AboutResponse {
        service: SERVICE_NAME,
        version: env!("CARGO_PKG_VERSION"),
        git_commit: option_env!("GIT_COMMIT_SHA"),
        production: observability::is_production(),
    })
}
