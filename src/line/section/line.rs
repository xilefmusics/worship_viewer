use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Line {
    pub chord: Option<String>,
    pub text: Option<String>,
    pub translation: Option<String>,
}
