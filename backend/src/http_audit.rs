//! Best-effort async persistence of one row per HTTP request (`http_request_audit`).

use std::future::{Ready, ready};
use std::rc::Rc;
use std::time::Instant;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use tracing::error;
use uuid::Uuid;

use crate::database::Database;
use crate::request_id::ApiRequestTarget;
use crate::resources::User;

/// Session id string for the authenticated request (set by [`crate::auth::middleware::RequireUser`]).
#[derive(Clone)]
pub struct AuditSessionId(pub String);

#[derive(Clone)]
pub struct HttpAudit {
    db: Data<Database>,
}

impl HttpAudit {
    pub fn new(db: Data<Database>) -> Self {
        Self { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for HttpAudit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = HttpAuditMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HttpAuditMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct HttpAuditMiddleware<S> {
    service: Rc<S>,
    db: Data<Database>,
}

impl<S, B> Service<ServiceRequest> for HttpAuditMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let db_data = self.db.clone();
        let started = Instant::now();
        let method = req.method().as_str().to_owned();
        let path_fallback = req
            .uri()
            .path_and_query()
            .map(|pq| pq.to_string())
            .unwrap_or_else(|| req.uri().path().to_owned());
        let request_id = req
            .extensions()
            .get::<String>()
            .cloned()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let api_path = req
            .extensions()
            .get::<ApiRequestTarget>()
            .map(|t| t.0.clone())
            .unwrap_or(path_fallback.clone());

        Box::pin(async move {
            let outcome = service.call(req).await;
            let duration_ms = started.elapsed().as_millis() as i64;

            let (status_code, user_id, session_id, path_for_row) = match &outcome {
                Ok(resp) => {
                    let r = resp.request();
                    let path = r
                        .extensions()
                        .get::<ApiRequestTarget>()
                        .map(|t| t.0.clone())
                        .unwrap_or_else(|| api_path.clone());
                    let user_id = r.extensions().get::<User>().map(|u| u.id.clone());
                    let session_id = r
                        .extensions()
                        .get::<AuditSessionId>()
                        .map(|s| s.0.clone());
                    (resp.status().as_u16() as i64, user_id, session_id, path)
                }
                Err(e) => (
                    actix_web::error::ResponseError::status_code(e.as_response_error()).as_u16()
                        as i64,
                    None,
                    None,
                    api_path,
                ),
            };

            let db_inner = db_data.clone();
            let row = HttpAuditInsert {
                request_id: request_id.clone(),
                method: method.clone(),
                path: path_for_row.clone(),
                status_code,
                duration_ms,
                user_id,
                session_id,
            };
            if cfg!(test) {
                insert_row(db_inner.get_ref(), row)
                    .await
                    .expect("http_request_audit insert (test)");
            } else {
                tokio::spawn(async move {
                    if let Err(e) = insert_row(db_inner.get_ref(), row).await {
                        error!(error = %e, "http_request_audit insert failed");
                    }
                });
            }

            outcome
        })
    }
}

struct HttpAuditInsert {
    request_id: String,
    method: String,
    path: String,
    status_code: i64,
    duration_ms: i64,
    user_id: Option<String>,
    session_id: Option<String>,
}

async fn insert_row(db: &Database, row: HttpAuditInsert) -> Result<(), surrealdb::Error> {
    let response = db
        .db
        .query(
            "CREATE http_request_audit SET request_id = $request_id, method = $method, \
             path = $path, status_code = $status_code, duration_ms = $duration_ms, \
             user = IF $user_id = NONE THEN NONE ELSE type::thing('user', $user_id) END, \
             session = IF $session_id = NONE THEN NONE ELSE type::thing('session', $session_id) END;",
        )
        .bind(("request_id", row.request_id))
        .bind(("method", row.method))
        .bind(("path", row.path))
        .bind(("status_code", row.status_code))
        .bind(("duration_ms", row.duration_ms))
        .bind(("user_id", row.user_id))
        .bind(("session_id", row.session_id))
        .await?;
    response.check()?;
    Ok(())
}
