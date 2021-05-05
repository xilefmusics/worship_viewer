#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Directive((String, String)),
    TextChordTrans(String),
    Empty,
}
