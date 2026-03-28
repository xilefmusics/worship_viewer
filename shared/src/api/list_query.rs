use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub q: Option<String>,
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

    pub fn with_q(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
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
        if let Some(ref q) = self.q {
            parts.push(format!("q={}", encode_query_value(q)));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

fn encode_query_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '&' => out.push_str("%26"),
            '=' => out.push_str("%3D"),
            '%' => out.push_str("%25"),
            '+' => out.push_str("%2B"),
            c => out.push(c),
        }
    }
    out
}
