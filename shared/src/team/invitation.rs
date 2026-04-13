use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::TeamUser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
pub struct TeamInvitation {
    pub id: String,
    pub team_id: String,
    pub created_by: TeamUser,
    pub created_at: DateTime<Utc>,
}
