use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderName;
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use uuid::Uuid;

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Middleware that generates a UUID v4 per request, stores it in the request
/// extensions as a `String`, and attaches it as an `X-Request-Id` response
/// header.
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
        let id = Uuid::new_v4().to_string();
        req.extensions_mut().insert(id.clone());

        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let mut resp = service.call(req).await?;
            resp.headers_mut()
                .insert(X_REQUEST_ID.clone(), id.parse().unwrap());
            Ok(resp)
        })
    }
}
