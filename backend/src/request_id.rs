use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderName;
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use uuid::Uuid;

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Request path and query (`/api/v1/...?a=1`), stored for diagnostics (e.g. Problem Details `instance`).
#[derive(Clone)]
pub struct ApiRequestTarget(pub String);

/// If `traceparent` is a valid W3C trace context value (`version-trace_id-span_id-flags`),
/// returns the 16-hex-character **span id** segment (often aligned with OpenTelemetry span id).
fn span_id_from_traceparent(traceparent: &str) -> Option<String> {
    let mut parts = traceparent.split('-');
    let version = parts.next()?;
    if version != "00" {
        return None;
    }
    let _trace_id = parts.next()?;
    let span_id = parts.next()?;
    let _flags = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    if span_id.len() != 16 || !span_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(span_id.to_ascii_lowercase())
}

/// Middleware that assigns `X-Request-Id`: prefers the span id from a valid `traceparent`
/// header (W3C trace context), otherwise a random UUID v4. The value is stored in request
/// extensions as a `String`.
#[derive(Clone, Default)]
pub struct RequestId;

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let id = req
            .headers()
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .and_then(span_id_from_traceparent)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        req.extensions_mut().insert(id.clone());
        let target = req
            .uri()
            .path_and_query()
            .map(|pq| pq.to_string())
            .unwrap_or_else(|| req.uri().path().to_string());
        req.extensions_mut().insert(ApiRequestTarget(target));

        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let mut resp = service.call(req).await?;
            resp.headers_mut()
                .insert(X_REQUEST_ID.clone(), id.parse().unwrap());
            Ok(resp)
        })
    }
}
