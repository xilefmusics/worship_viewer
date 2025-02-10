use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultCollectionLink {
    pub id: Option<String>,
    // TODO: save as Record field
    pub collection_id: String,
}

impl fancy_surreal::Databasable for DefaultCollectionLink {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
