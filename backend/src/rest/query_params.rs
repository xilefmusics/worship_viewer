use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}
