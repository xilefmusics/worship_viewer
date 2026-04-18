//! Weak ETags and conditional GET helpers (P2 REST review).

use actix_web::HttpRequest;
use actix_web::http::header::{IF_MATCH, IF_NONE_MATCH};
use ring::digest::{SHA256, digest};

use crate::error::AppError;

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

/// Returns true when the request's `If-Match` includes `*`, or any listed value that matches
/// `current_weak_etag` after weak ETag normalization.
pub fn if_match_matches(req: &HttpRequest, current_weak_etag: &str) -> bool {
    let Some(hdr) = req.headers().get(IF_MATCH) else {
        return false;
    };
    let Ok(raw) = hdr.to_str() else {
        return false;
    };
    let server = normalize_etag(current_weak_etag);
    for part in raw.split(',') {
        let client = normalize_etag(part);
        if client == "*" {
            return true;
        }
        if client == server {
            return true;
        }
    }
    false
}

/// When `If-Match` is absent, allows the request. When present, it must match `current_weak_etag`
/// (same value as [`weak_etag_json`] on the current GET body) or the handler returns **412**.
pub fn check_if_match(req: &HttpRequest, current_weak_etag: &str) -> Result<(), AppError> {
    if !req.headers().contains_key(IF_MATCH) {
        return Ok(());
    }
    if if_match_matches(req, current_weak_etag) {
        Ok(())
    } else {
        Err(AppError::precondition_failed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use actix_web::http::header::IF_MATCH;
    use actix_web::test::TestRequest;

    #[test]
    fn check_if_match_allows_when_header_absent() {
        let req = TestRequest::default().to_http_request();
        assert!(check_if_match(&req, r#"W/\"abc\""#).is_ok());
    }

    #[test]
    fn check_if_match_accepts_normalized_match() {
        let etag = r#"W/\"7f8e9a\""#;
        let req = TestRequest::default()
            .insert_header((IF_MATCH, etag))
            .to_http_request();
        assert!(check_if_match(&req, etag).is_ok());
    }

    #[test]
    fn check_if_match_star_accepts() {
        let req = TestRequest::default()
            .insert_header((IF_MATCH, "*"))
            .to_http_request();
        assert!(check_if_match(&req, r#"W/\"anything\""#).is_ok());
    }

    #[test]
    fn check_if_match_rejects_stale() {
        let req = TestRequest::default()
            .insert_header((IF_MATCH, r#"W/\"old\""#))
            .to_http_request();
        let r = check_if_match(&req, r#"W/\"new\""#);
        assert!(matches!(r, Err(AppError::PreconditionFailed)));
    }
}
