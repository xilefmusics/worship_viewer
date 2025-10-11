use crate::song::Filter;
use serde::Deserialize;
use super::Format;

#[derive(Debug, Deserialize, Clone)]
pub struct QueryParams {
    pub id: Option<String>,
    pub collection: Option<String>,
    pub setlist: Option<String>,
    #[serde(default)]
    pub format: Format,
}

impl QueryParams {
    pub fn to_filter<'a>(&'a self) -> Filter<'a> {
        Filter::new(
            self.id.as_deref(),
            self.collection.as_deref(),
            self.setlist.as_deref(),
        )
    }
}