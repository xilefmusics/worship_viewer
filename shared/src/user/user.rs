use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Option<String>,
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[cfg(feature = "backend")]
impl fancy_surreal::Databasable for User {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
