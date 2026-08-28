//! HTTP byte-range parsing and response construction for authenticated asset delivery.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use actix_web::HttpRequest;
use actix_web::body::BodyStream;
use actix_web::http::StatusCode;
use actix_web::http::header::{
    ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, HeaderValue,
    IF_RANGE, RANGE,
};
use actix_web::{HttpResponse, web};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::error::AppError;
use crate::http_cache::{if_none_match_matches, weak_etag_from_bytes};

const CHUNK_SIZE: usize = 64 * 1024;
const X_CONTENT_TYPE_OPTIONS: &str = "x-content-type-options";

/// Parsed single HTTP range against a file of `total_len` bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn length(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }
}

/// Result of parsing a `Range` request header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeParseResult {
    /// No `Range` header — serve the full entity.
    Full,
    /// Single satisfiable byte range.
    Satisfiable(ByteRange),
    /// Unsatisfiable range (empty file or out of bounds).
    Unsatisfiable,
    /// Multiple ranges or malformed range syntax.
    Invalid,
}

/// Parse a `bytes=` range header value for a file of `total_len` bytes.
pub fn parse_range_header(value: &str, total_len: u64) -> RangeParseResult {
    let trimmed = value.trim();
    if !trimmed.starts_with("bytes=") {
        return RangeParseResult::Invalid;
    }
    let spec = &trimmed[6..];
    if spec.is_empty() {
        return RangeParseResult::Invalid;
    }
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    if parts.len() != 1 {
        return RangeParseResult::Invalid;
    }
    parse_single_range(parts[0], total_len)
}

fn parse_single_range(part: &str, total_len: u64) -> RangeParseResult {
    if total_len == 0 {
        return RangeParseResult::Unsatisfiable;
    }
    if let Some(suffix) = part.strip_prefix('-') {
        let suffix_len: u64 = match suffix.parse() {
            Ok(v) => v,
            Err(_) => return RangeParseResult::Invalid,
        };
        if suffix_len == 0 {
            return RangeParseResult::Invalid;
        }
        let start = total_len.saturating_sub(suffix_len);
        return RangeParseResult::Satisfiable(ByteRange {
            start,
            end: total_len - 1,
        });
    }
    let (start_str, end_str) = match part.split_once('-') {
        Some(pair) => pair,
        None => return RangeParseResult::Invalid,
    };
    let start: u64 = match start_str.parse() {
        Ok(v) => v,
        Err(_) => return RangeParseResult::Invalid,
    };
    if start >= total_len {
        return RangeParseResult::Unsatisfiable;
    }
    let end = if end_str.is_empty() {
        total_len - 1
    } else {
        match end_str.parse::<u64>() {
            Ok(v) => v.min(total_len - 1),
            Err(_) => return RangeParseResult::Invalid,
        }
    };
    if start > end {
        return RangeParseResult::Invalid;
    }
    RangeParseResult::Satisfiable(ByteRange { start, end })
}

fn if_range_matches(req: &HttpRequest, etag: &str) -> bool {
    let Some(hdr) = req.headers().get(IF_RANGE) else {
        return true;
    };
    let Ok(raw) = hdr.to_str() else {
        return false;
    };
    let client = raw
        .trim()
        .trim_start_matches("W/")
        .trim_matches('"')
        .to_string();
    let server = etag
        .trim()
        .trim_start_matches("W/")
        .trim_matches('"')
        .to_string();
    client == server
}

fn private_cache_headers() -> HeaderValue {
    HeaderValue::from_static("private, max-age=3600, immutable")
}

fn content_range_value(start: u64, end: u64, total: u64) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(&format!("bytes {start}-{end}/{total}"))
        .map_err(|e| AppError::internal_from_err("http_range.content_range", e))
}

fn unsatisfiable_range_response(total_len: u64) -> Result<HttpResponse, AppError> {
    let cr = HeaderValue::from_str(&format!("bytes */{total_len}"))
        .map_err(|e| AppError::internal_from_err("http_range.unsatisfiable", e))?;
    Ok(HttpResponse::build(StatusCode::RANGE_NOT_SATISFIABLE)
        .insert_header((CONTENT_RANGE, cr))
        .insert_header((ACCEPT_RANGES, HeaderValue::from_static("bytes")))
        .finish())
}

fn file_stream_from(
    path: PathBuf,
    start: u64,
    length: u64,
) -> BodyStream<impl futures_util::Stream<Item = Result<web::Bytes, actix_web::Error>>> {
    let stream = async_stream::stream! {
        let mut file = match File::open(&path).await {
            Ok(f) => f,
            Err(e) => {
                tracing::error!(error = %e, "http_range.open_failed");
                return;
            }
        };
        if file.seek(SeekFrom::Start(start)).await.is_err() {
            return;
        }
        let mut remaining = length;
        let mut buf = vec![0u8; CHUNK_SIZE];
        while remaining > 0 {
            let to_read = (remaining as usize).min(CHUNK_SIZE);
            match file.read(&mut buf[..to_read]).await {
                Ok(0) => break,
                Ok(n) => {
                    remaining -= n as u64;
                    yield Ok::<_, actix_web::Error>(web::Bytes::copy_from_slice(&buf[..n]));
                }
                Err(e) => {
                    tracing::error!(error = %e, "http_range.read_failed");
                    break;
                }
            }
        }
    };
    BodyStream::new(stream)
}

/// Build a GET or HEAD response for a file with range and conditional semantics.
pub async fn file_data_response(
    req: &HttpRequest,
    path: &Path,
    content_type: &str,
    etag: &str,
    head_only: bool,
) -> Result<HttpResponse, AppError> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|_| AppError::NotFound("asset not found".into()))?;
    let total_len = metadata.len();

    if if_none_match_matches(req, etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((
                ETAG,
                HeaderValue::from_str(etag)
                    .map_err(|e| AppError::internal_from_err("http_range.etag", e))?,
            ))
            .insert_header((CACHE_CONTROL, private_cache_headers()))
            .finish());
    }

    let range_result = match req.headers().get(RANGE) {
        Some(hdr) => {
            let Ok(raw) = hdr.to_str() else {
                return unsatisfiable_range_response(total_len);
            };
            parse_range_header(raw, total_len)
        }
        None => RangeParseResult::Full,
    };

    let ct = HeaderValue::from_str(content_type)
        .map_err(|e| AppError::internal_from_err("http_range.content_type", e))?;
    let etag_hdr = HeaderValue::from_str(etag)
        .map_err(|e| AppError::internal_from_err("http_range.etag", e))?;

    match range_result {
        RangeParseResult::Full => {
            let mut builder = HttpResponse::Ok();
            builder
                .insert_header((ETAG, etag_hdr))
                .insert_header((CONTENT_TYPE, ct))
                .insert_header((X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .insert_header((CACHE_CONTROL, private_cache_headers()))
                .insert_header((ACCEPT_RANGES, HeaderValue::from_static("bytes")))
                .insert_header((CONTENT_LENGTH, total_len.to_string()));
            if head_only {
                return Ok(builder.finish());
            }
            Ok(builder.body(file_stream_from(path.to_path_buf(), 0, total_len)))
        }
        RangeParseResult::Satisfiable(range) => {
            if !if_range_matches(req, etag) {
                let mut builder = HttpResponse::Ok();
                builder
                    .insert_header((ETAG, etag_hdr))
                    .insert_header((CONTENT_TYPE, ct))
                    .insert_header((X_CONTENT_TYPE_OPTIONS, "nosniff"))
                    .insert_header((CACHE_CONTROL, private_cache_headers()))
                    .insert_header((ACCEPT_RANGES, HeaderValue::from_static("bytes")))
                    .insert_header((CONTENT_LENGTH, total_len.to_string()));
                if head_only {
                    return Ok(builder.finish());
                }
                return Ok(builder.body(file_stream_from(path.to_path_buf(), 0, total_len)));
            }
            let len = range.length();
            let cr = content_range_value(range.start, range.end, total_len)?;
            let mut builder = HttpResponse::PartialContent();
            builder
                .insert_header((ETAG, etag_hdr))
                .insert_header((CONTENT_TYPE, ct))
                .insert_header((X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .insert_header((CACHE_CONTROL, private_cache_headers()))
                .insert_header((ACCEPT_RANGES, HeaderValue::from_static("bytes")))
                .insert_header((CONTENT_RANGE, cr))
                .insert_header((CONTENT_LENGTH, len.to_string()));
            if head_only {
                return Ok(builder.finish());
            }
            Ok(builder.body(file_stream_from(path.to_path_buf(), range.start, len)))
        }
        RangeParseResult::Unsatisfiable | RangeParseResult::Invalid => {
            unsatisfiable_range_response(total_len)
        }
    }
}

/// Compute a weak ETag from file bytes (used during promotion).
pub fn etag_from_file_bytes(bytes: &[u8]) -> String {
    weak_etag_from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::http::header::IF_NONE_MATCH;
    use actix_web::{App, HttpResponse, web};
    use tempfile::TempDir;

    #[test]
    fn parse_first_byte_range() {
        match parse_range_header("bytes=0-0", 100) {
            RangeParseResult::Satisfiable(r) => {
                assert_eq!(r.start, 0);
                assert_eq!(r.end, 0);
            }
            other => panic!("expected satisfiable, got {other:?}"),
        }
    }

    #[test]
    fn parse_open_ended_range() {
        match parse_range_header("bytes=10-", 100) {
            RangeParseResult::Satisfiable(r) => {
                assert_eq!(r.start, 10);
                assert_eq!(r.end, 99);
            }
            other => panic!("expected satisfiable, got {other:?}"),
        }
    }

    #[test]
    fn parse_suffix_range() {
        match parse_range_header("bytes=-10", 100) {
            RangeParseResult::Satisfiable(r) => {
                assert_eq!(r.start, 90);
                assert_eq!(r.end, 99);
            }
            other => panic!("expected satisfiable, got {other:?}"),
        }
    }

    #[test]
    fn parse_multiple_ranges_invalid() {
        assert_eq!(
            parse_range_header("bytes=0-1,2-3", 100),
            RangeParseResult::Invalid
        );
    }

    #[test]
    fn parse_out_of_bounds_unsatisfiable() {
        assert_eq!(
            parse_range_header("bytes=200-", 100),
            RangeParseResult::Unsatisfiable
        );
    }

    #[test]
    fn parse_empty_file_unsatisfiable() {
        assert_eq!(
            parse_range_header("bytes=0-", 0),
            RangeParseResult::Unsatisfiable
        );
    }

    #[actix_web::test]
    async fn full_get_and_head() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("data.bin");
        std::fs::write(&path, b"hello world").unwrap();
        let etag = etag_from_file_bytes(b"hello world");
        let test_data = web::Data::new((path.clone(), etag));

        async fn handler(
            req: HttpRequest,
            data: web::Data<(std::path::PathBuf, String)>,
        ) -> Result<HttpResponse, AppError> {
            let (path, etag) = data.get_ref();
            file_data_response(&req, path, "application/octet-stream", etag, false).await
        }

        async fn head_handler(
            req: HttpRequest,
            data: web::Data<(std::path::PathBuf, String)>,
        ) -> Result<HttpResponse, AppError> {
            let (path, etag) = data.get_ref();
            file_data_response(&req, path, "application/octet-stream", etag, true).await
        }

        let app = actix_web::test::init_service(
            App::new()
                .app_data(test_data.clone())
                .route("/", web::get().to(handler))
                .route("/head", web::head().to(head_handler)),
        )
        .await;

        let req = actix_web::test::TestRequest::get().uri("/").to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );
        let body = actix_web::test::read_body(resp).await;
        assert_eq!(&body[..], b"hello world");

        let req = actix_web::test::TestRequest::default()
            .method(actix_web::http::Method::HEAD)
            .uri("/head")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = actix_web::test::read_body(resp).await;
        assert!(body.is_empty());
    }

    #[actix_web::test]
    async fn partial_content_206() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("data.bin");
        std::fs::write(&path, b"0123456789").unwrap();
        let test_data = web::Data::new((path, etag_from_file_bytes(b"0123456789")));

        async fn handler(
            req: HttpRequest,
            data: web::Data<(std::path::PathBuf, String)>,
        ) -> Result<HttpResponse, AppError> {
            let (path, etag) = data.get_ref();
            file_data_response(&req, path, "application/octet-stream", etag, false).await
        }

        let app = actix_web::test::init_service(
            App::new()
                .app_data(test_data)
                .route("/", web::get().to(handler)),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/")
            .insert_header((RANGE, "bytes=2-5"))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
        let body = actix_web::test::read_body(resp).await;
        assert_eq!(&body[..], b"2345");
    }

    #[actix_web::test]
    async fn if_none_match_304() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("data.bin");
        std::fs::write(&path, b"data").unwrap();
        let etag = etag_from_file_bytes(b"data");
        let test_data = web::Data::new((path, etag.clone()));

        async fn handler(
            req: HttpRequest,
            data: web::Data<(std::path::PathBuf, String)>,
        ) -> Result<HttpResponse, AppError> {
            let (path, etag) = data.get_ref();
            file_data_response(&req, path, "application/octet-stream", etag, false).await
        }

        let app = actix_web::test::init_service(
            App::new()
                .app_data(test_data)
                .route("/", web::get().to(handler)),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/")
            .insert_header((IF_NONE_MATCH, etag.as_str()))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
    }

    #[actix_web::test]
    async fn unsatisfiable_range_416() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("data.bin");
        std::fs::write(&path, b"short").unwrap();
        let test_data = web::Data::new((path, etag_from_file_bytes(b"short")));

        async fn handler(
            req: HttpRequest,
            data: web::Data<(std::path::PathBuf, String)>,
        ) -> Result<HttpResponse, AppError> {
            let (path, etag) = data.get_ref();
            file_data_response(&req, path, "application/octet-stream", etag, false).await
        }

        let app = actix_web::test::init_service(
            App::new()
                .app_data(test_data)
                .route("/", web::get().to(handler)),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/")
            .insert_header((RANGE, "bytes=100-"))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    }
}
