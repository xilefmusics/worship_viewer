use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct TocItem {
    pub idx: usize,
    pub title: String,
    pub id: Option<String>,
    pub nr: String,
    pub liked: bool,
}
