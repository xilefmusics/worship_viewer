//! RFC 5988 `Link` helpers for paginated JSON list responses.

use actix_web::HttpRequest;
use actix_web::http::header;

use shared::api::pagination_link_header;

pub fn request_origin(req: &HttpRequest) -> String {
    let c = req.connection_info();
    format!("{}://{}", c.scheme(), c.host())
}

/// Build a `Link` `HeaderValue` for the current path and pagination state.
pub fn list_link_header(
    req: &HttpRequest,
    query_for_page: impl Fn(u32) -> String,
    current_page: u32,
    page_size: u32,
    total: u64,
) -> header::HeaderValue {
    let v = pagination_link_header(
        &request_origin(req),
        req.path(),
        query_for_page,
        current_page,
        page_size,
        total,
    );
    header::HeaderValue::from_str(&v).expect("RFC 5988 Link header ASCII")
}
