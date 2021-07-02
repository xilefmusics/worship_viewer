#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Keyword(String),
    Chord(String),
    Text(String),
    Translation(String),
    Comment(String),
}
