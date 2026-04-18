//! RFC 5988 pagination `Link` header (`rel=first|prev|next|last`).

use super::list_query::PAGE_SIZE_DEFAULT;

/// Build a `Link` header value for offset pagination (`page` is zero-based).
///
/// `path` is the absolute path (e.g. `/api/v1/songs`). `query_for_page` must return the
/// **full** query string for the given page **without** a leading `?` (e.g. `page=1&page_size=50&q=x`).
pub fn pagination_link_header(
    origin: &str,
    path: &str,
    query_for_page: impl Fn(u32) -> String,
    current_page: u32,
    page_size_raw: u32,
    total: u64,
) -> String {
    let page_size = if page_size_raw == 0 {
        PAGE_SIZE_DEFAULT
    } else {
        page_size_raw
    };
    let ps = page_size as u64;
    let last_page = if total == 0 {
        0
    } else {
        (total.saturating_sub(1) / ps) as u32
    };

    let mut out: Vec<String> = Vec::with_capacity(4);
    let mut push = |rel: &'static str, p: u32| {
        let qs = query_for_page(p);
        let url = if qs.is_empty() {
            format!("{origin}{path}")
        } else {
            format!("{origin}{path}?{qs}")
        };
        out.push(format!("<{url}>; rel=\"{rel}\""));
    };

    push("first", 0);
    if current_page > 0 {
        push("prev", current_page - 1);
    }
    if current_page < last_page {
        push("next", current_page + 1);
    }
    push("last", last_page);
    out.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_header_middle_page_has_four_rels() {
        let h = pagination_link_header(
            "https://ex.test",
            "/api/v1/songs",
            |p| format!("page={p}&page_size=10"),
            3,
            10,
            100,
        );
        assert!(h.contains("rel=\"first\""));
        assert!(h.contains("rel=\"prev\""));
        assert!(h.contains("rel=\"next\""));
        assert!(h.contains("rel=\"last\""));
        assert!(h.contains("page=0"));
        assert!(h.contains("page=2"));
        assert!(h.contains("page=4"));
        assert!(h.contains("page=9"));
    }

    #[test]
    fn link_header_first_page_omits_prev() {
        let h = pagination_link_header(
            "https://ex.test",
            "/api/v1/x",
            |p| format!("page={p}"),
            0,
            10,
            50,
        );
        assert!(!h.contains("rel=\"prev\""));
        assert!(h.contains("rel=\"next\""));
    }

    #[test]
    fn link_header_last_page_omits_next() {
        let h = pagination_link_header(
            "https://ex.test",
            "/api/v1/x",
            |p| format!("page={p}"),
            4,
            10,
            50,
        );
        assert!(!h.contains("rel=\"next\""));
        assert!(h.contains("rel=\"prev\""));
    }
}
