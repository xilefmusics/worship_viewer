use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdWrapper<T> {
    pub id: String,
    pub data: T,
}
