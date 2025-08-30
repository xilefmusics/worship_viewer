use super::Filter;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct QueryParams {
    pub id: Option<String>,
    pub collection: Option<String>,
    pub setlist: Option<String>,
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
