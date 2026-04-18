use crate::auth::middleware::RequireAdmin;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::request_link;
use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, get,
    web::{Data, Query},
};
use shared::api::{PageQuery, PAGE_SIZE_DEFAULT};

use super::repo::MonitoringRepo;

pub fn scope() -> Scope {
    actix_web::web::scope("/monitoring").service(
        actix_web::web::scope("")
            .wrap(RequireAdmin)
            .service(list_http_audit_logs),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/monitoring/http-audit-logs",
    params(
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Defaults to 0.", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50.", minimum = 1, maximum = 500, example = 50, nullable = true),
    ),
    responses(
        (status = 200, description = "Paginated HTTP request audit log (newest first). `X-Total-Count` is the total row count.", body = [crate::resources::monitoring::HttpAuditLog]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Admin role required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to list audit logs", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Monitoring",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/http-audit-logs")]
async fn list_http_audit_logs(
    req: HttpRequest,
    db: Data<Database>,
    query: Query<PageQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let list_query = query.as_list_query();
    let q_link = list_query.clone();
    let page = list_query.page.unwrap_or(0);
    let page_size = list_query.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let items = MonitoringRepo::list_http_audit_logs(db.get_ref(), list_query.clone()).await?;
    let total = MonitoringRepo::count_http_audit_logs(db.get_ref()).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .insert_header((
            header::LINK,
            request_link::list_link_header(
                &req,
                |p| q_link.query_string_for_page(p),
                page,
                page_size,
                total,
            ),
        ))
        .json(items))
}
