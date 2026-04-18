use actix_web::http::header;
use actix_web::{
    HttpResponse, delete, get, post,
    web::{Data, Path, Query, ReqData},
};
use serde::Deserialize;

use shared::api::{ListQuery, PageQuery};

#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::resources::User;
use crate::settings::CookieConfig;

use super::service::SessionServiceHandle;

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions",
    params(
        ("page" = Option<u32>, Query, description = "Zero-based page. With `page_size`, `X-Total-Count` is pre-pagination total (`list-pagination.md`).", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50. Omit with `page` for full list.", minimum = 1, maximum = 500, example = 50, nullable = true),
    ),
    responses(
        (status = 200, description = "Returns active sessions for the current user. `X-Total-Count` is the total before paging.", body = [Session]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to list sessions for current user", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/me/sessions")]
pub async fn get_sessions_for_current_user(
    svc: Data<SessionServiceHandle>,
    user: ReqData<User>,
    query: Query<PageQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let sessions = svc.get_sessions_by_user_id(&user.id).await?;
    let lq = query.as_list_query();
    let (page, total) = ListQuery::paginate_nested_vec(sessions, &lq);
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(page))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session identifier")
    ),
    responses(
        (status = 200, description = "Returns a session for the current user", body = Session),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Session not found for current user", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/me/sessions/{id}")]
pub async fn get_session_for_current_user(
    svc: Data<SessionServiceHandle>,
    user: ReqData<User>,
    path: Path<SessionPath>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_session_for_user(&path.id, &user.id).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/me/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session identifier")
    ),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Session not found for current user", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to delete session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/me/sessions/{id}")]
pub async fn delete_session_for_current_user(
    svc: Data<SessionServiceHandle>,
    user: ReqData<User>,
    path: Path<SessionPath>,
) -> Result<HttpResponse, AppError> {
    svc.delete_session_for_user(&path.id, &user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 201, description = "Creates a session for the specified user", body = Session),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Admin role required", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to create session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("/{user_id}/sessions")]
pub async fn create_session_for_user(
    svc: Data<SessionServiceHandle>,
    cookie_cfg: Data<CookieConfig>,
    path: Path<UserIdPath>,
) -> Result<HttpResponse, AppError> {
    let ttl = cookie_cfg.session_ttl_seconds as i64;
    Ok(HttpResponse::Created().json(
        svc.create_session_for_user_by_id(&path.user_id, ttl)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("page" = Option<u32>, Query, description = "Zero-based page. With `page_size`, `X-Total-Count` is pre-pagination total (`list-pagination.md`).", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50. Omit with `page` for full list.", minimum = 1, maximum = 500, example = 50, nullable = true),
    ),
    responses(
        (status = 200, description = "Returns active sessions for the specified user. `X-Total-Count` is the total before paging.", body = [Session]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Admin role required", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to list sessions", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{user_id}/sessions")]
pub async fn get_sessions_for_user(
    svc: Data<SessionServiceHandle>,
    path: Path<UserIdPath>,
    query: Query<PageQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let sessions = svc.get_sessions_by_user_id(&path.user_id).await?;
    let lq = query.as_list_query();
    let (page, total) = ListQuery::paginate_nested_vec(sessions, &lq);
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(page))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/sessions/{id}",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("id" = String, Path, description = "Session identifier")
    ),
    responses(
        (status = 200, description = "Returns a session for the specified user", body = Session),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Admin role required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Session not found for specified user", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{user_id}/sessions/{id}")]
pub async fn get_session_for_user(
    svc: Data<SessionServiceHandle>,
    path: Path<UserSessionPath>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_session_for_user(&path.id, &path.user_id).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/sessions/{id}",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("id" = String, Path, description = "Session identifier")
    ),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Admin role required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Session not found for specified user", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to delete session", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{user_id}/sessions/{id}")]
pub async fn delete_session_for_user(
    svc: Data<SessionServiceHandle>,
    path: Path<UserSessionPath>,
) -> Result<HttpResponse, AppError> {
    svc.delete_session_for_user(&path.id, &path.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize)]
struct SessionPath {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct UserIdPath {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
struct UserSessionPath {
    user_id: String,
    id: String,
}
