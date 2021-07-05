#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Keyword(String),
    Chord(String),
    Text(String),
    TranslationChord(String),
    TranslationText(String),
    Comment(String),
}
