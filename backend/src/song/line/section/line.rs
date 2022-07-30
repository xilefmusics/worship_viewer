use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub chord: Option<String>,
    pub text: Option<String>,
    pub translation_chord: Option<String>,
    pub translation_text: Option<String>,
    pub comment: Option<String>,
}
