use crate::route::Route;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Query {
    pub id: Option<String>,
    pub collection: Option<String>,
    pub setlist: Option<String>,
}

impl Query {
    pub fn back_route(&self) -> Route {
        if self.setlist.is_some() {
            Route::Setlists
        } else if self.collection.is_some() {
            Route::Collections
        } else if self.id.is_some() {
            Route::Songs
        } else {
            Route::NotFound
        }
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        if let Some(setlist) = self.setlist.as_ref() {
            HashMap::from([("setlist".to_string(), setlist.to_owned())])
        } else if let Some(collection) = self.collection.as_ref() {
            HashMap::from([("collection".to_string(), collection.to_owned())])
        } else if let Some(id) = self.id.as_ref() {
            HashMap::from([("id".to_string(), id.to_owned())])
        } else {
            HashMap::new()
        }
    }
}