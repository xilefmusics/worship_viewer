use serde::{Deserialize, Serialize};

use super::Line;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub keyword: Option<String>,
    pub lines: Vec<Line>,
}
