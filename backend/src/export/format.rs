use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub enum Format {
    #[default]
    WorshipPro,
    ChordPro,
    Pdf,
}