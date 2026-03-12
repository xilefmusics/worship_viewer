use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl ListQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn to_offset_limit(&self) -> Option<(u32, u32)> {
        match (self.page, self.page_size) {
            (Some(page), Some(page_size)) if page_size > 0 => {
                let offset = page.saturating_mul(page_size);
                Some((offset, page_size))
            }
            _ => None,
        }
    }

    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(page) = self.page {
            parts.push(format!("page={}", page));
        }
        if let Some(page_size) = self.page_size {
            parts.push(format!("page_size={}", page_size));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

