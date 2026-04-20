//! Request body for `POST .../{id}/move` (change owning team).

#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;
#[cfg(feature = "backend")]
use utoipa::ToSchema;

use serde::{Deserialize, Serialize};

/// Target owning team for a library resource move (`owner` string matches GET responses).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({ "owner": "team_target_id" }))
)]
pub struct MoveOwner {
    pub owner: String,
}
