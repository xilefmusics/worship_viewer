//! Weak ETags and conditional GET helpers (P2 REST review).

use actix_web::HttpRequest;
use actix_web::http::header::IF_NONE_MATCH;
use ring::digest::{SHA256, digest};

pub fn weak_etag_from_bytes(bytes: &[u8]) -> String {
    let d = digest(&SHA256, bytes);
    format!("W/\"{}\"", hex::encode(d.as_ref()))
}

pub fn weak_etag_json<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(v)?;
    Ok(weak_etag_from_bytes(&bytes))
}

fn normalize_etag(s: &str) -> String {
    s.trim()
        .trim_start_matches("W/")
        .trim_matches('"')
        .to_string()
}

/// Returns true when the request's `If-None-Match` matches `etag` (weak comparison).
pub fn if_none_match_matches(req: &HttpRequest, etag: &str) -> bool {
    let Some(hdr) = req.headers().get(IF_NONE_MATCH) else {
        return false;
    };
    let Ok(raw) = hdr.to_str() else {
        return false;
    };
    let server = normalize_etag(etag);
    for part in raw.split(',') {
        let client = normalize_etag(part);
        if client == "*" || client == server {
            return true;
        }
    }
    false
}
