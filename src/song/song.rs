use serde::{Deserialize, Serialize};

use super::line::{IterExtSectionToWp, IterExtToString, Section, SectionToWp, ToString, WpLine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub key: String,
    pub sections: Vec<Section>,
}

impl Song {
    pub fn to_wp(
        &self,
    ) -> std::iter::Chain<std::vec::IntoIter<WpLine>, SectionToWp<std::vec::IntoIter<Section>>>
    {
        vec![
            WpLine::Directive(("title".to_string(), self.title.clone())),
            WpLine::Directive(("artist".to_string(), self.artist.clone())),
            WpLine::Directive(("key".to_string(), self.key.clone())),
        ]
        .into_iter()
        .chain(self.sections.clone().into_iter().to_wp())
    }
    pub fn to_string(
        &self,
    ) -> ToString<
        std::iter::Chain<std::vec::IntoIter<WpLine>, SectionToWp<std::vec::IntoIter<Section>>>,
    > {
        self.to_wp().to_string()
    }

    pub fn has_translation(&self) -> bool {
        for section in &self.sections {
            for line in &section.lines {
                if line.translation_text.is_some() {
                    return true;
                }
            }
        }
        false
    }
}
