use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct TocItem {
    pub idx: usize,
    pub title: String,
    pub id: Option<String>,
    pub nr: String,
}
