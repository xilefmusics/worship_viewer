use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Like {
    pub id: Option<String>,
    pub song: String,
}
