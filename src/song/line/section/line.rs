use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub chord: Option<String>,
    pub text: Option<String>,
    pub translation: Option<String>,
}
