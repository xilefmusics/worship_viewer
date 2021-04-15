#[derive(Debug, Clone)]
pub enum Line {
    Directive((String, String)),
    TextChordTrans(String),
    Empty,
}
