use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: Option<String>,
    pub read: Vec<String>,
    pub write: Vec<String>,
}