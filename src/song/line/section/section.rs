use serde::Serialize;

use super::Line;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Section {
    pub keyword: Option<String>,
    pub lines: Vec<Line>,
}
