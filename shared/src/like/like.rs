use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Like {
    pub id: Option<String>,
    pub song: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct LikeStatus {
    pub liked: bool,
}
