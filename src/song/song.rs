use serde::{Deserialize, Serialize};

use super::line::{IterExtSectionToWp, IterExtToString, Section, SectionToWp, ToString, WpLine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub key: String,
    pub sections: Vec<Section>,
}

impl Song {
    pub fn to_wp(
        &self,
    ) -> std::iter::Chain<
        std::iter::Chain<std::iter::Once<WpLine>, std::iter::Once<WpLine>>,
        SectionToWp<std::vec::IntoIter<Section>>,
    > {
        std::iter::once(WpLine::Directive(("title".to_string(), self.title.clone())))
            .chain(std::iter::once(WpLine::Directive((
                "key".to_string(),
                self.key.clone(),
            ))))
            .chain(self.sections.clone().into_iter().to_wp())
    }
    pub fn to_string(
        &self,
    ) -> ToString<
        std::iter::Chain<
            std::iter::Chain<std::iter::Once<WpLine>, std::iter::Once<WpLine>>,
            SectionToWp<std::vec::IntoIter<Section>>,
        >,
    > {
        self.to_wp().to_string()
    }
}
