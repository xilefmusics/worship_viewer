use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, delete, get, post,
    web::{Data, Path, Query, ReqData},
};
use serde::Deserialize;

use shared::api::{ListQuery, PAGE_SIZE_DEFAULT, PageQuery};
use shared::user::SessionBody;

#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::expand::expand_includes_user;
use crate::resources::User;
use crate::settings::CookieConfig;

use super::service::SessionServiceHandle;

#[derive(Debug, Deserialize)]
struct SessionsPageQuery {
    #[serde(flatten)]
    page: PageQuery,
    /// Comma-separated relations to expand. Use `user` to embed the full [`crate::resources::User`] instead of the default `id`+`email` link.
    expand: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExpandQuery {
    /// Comma-separated relations to expand (`user` → full user object on `user`).
    expand: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions",
    params(
        ("page" = Option<u32>, Query, description = "Zero-based page; defaults to 0. `X-Total-Count` is the total before paging (`list-pagination.md`).", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50 when omitted.", minimum = 1, maximum = 500, example = 50, nullable = true),
        ("expand" = Option<String>, Query, description = "Optional comma-separated relations (`user` = embed full user on each session; default is `id`+`email` link only)."),
    ),
    responses(
        (status = 200, description = "Returns active sessions for the current user. `X-Total-Count` is the total before paging.", body = [SessionBody]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    req: HttpRequest,
    svc: Data<SessionServiceHandle>,
    user: ReqData<User>,
    query: Query<SessionsPageQuery>,
) -> Result<HttpResponse, AppError> {
    let SessionsPageQuery { page, expand } = query.into_inner();
    let page = page
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let expand_user = expand_includes_user(&expand);
    let q_link = page.clone();
    let cur_page = page.page.unwrap_or(0);
    let page_size = page.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let sessions = svc.get_sessions_by_user_id(&user.id).await?;
    let lq = page.as_list_query();
    let (sessions_page, total) = ListQuery::paginate_vec(sessions, &lq);
    let sessions_page: Vec<SessionBody> = sessions_page
        .into_iter()
        .map(|s| SessionBody::from_session(s, expand_user))
        .collect();
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .insert_header((
            header::LINK,
            crate::request_link::list_link_header(
                &req,
                |p| q_link.query_string_for_page(p),
                cur_page,
                page_size,
                total,
            ),
        ))
        .json(sessions_page))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session identifier"),
        ("expand" = Option<String>, Query, description = "Optional `user` to embed full user (default: `id`+`email` link)."),
    ),
    responses(
        (status = 200, description = "Returns a session for the current user", body = SessionBody),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    expand: Query<ExpandQuery>,
) -> Result<HttpResponse, AppError> {
    let session = svc.get_session_for_user(&path.id, &user.id).await?;
    Ok(HttpResponse::Ok().json(SessionBody::from_session(
        session,
        expand_includes_user(&expand.expand),
    )))
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
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    let user = user.into_inner();
    let deleted = svc.delete_session_for_user(&path.id, &user.id).await?;
    crate::audit!(
        "audit.session.revoked",
        session_id = tracing::field::display(&deleted.id),
        user_id = tracing::field::display(&deleted.user.id),
        actor_user_id = tracing::field::display(&user.id)
        ; "session revoked"
    );
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("expand" = Option<String>, Query, description = "Optional `user` to embed full user in the response (default: link)."),
    ),
    responses(
        (status = 201, description = "Creates a session for the specified user", body = SessionBody),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    expand: Query<ExpandQuery>,
) -> Result<HttpResponse, AppError> {
    let ttl = cookie_cfg.session_ttl_seconds as i64;
    let session = svc
        .create_session_for_user_by_id(&path.user_id, ttl)
        .await?;
    Ok(HttpResponse::Created().json(SessionBody::from_session(
        session,
        expand_includes_user(&expand.expand),
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("page" = Option<u32>, Query, description = "Zero-based page; defaults to 0. `X-Total-Count` is the total before paging (`list-pagination.md`).", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50 when omitted.", minimum = 1, maximum = 500, example = 50, nullable = true),
        ("expand" = Option<String>, Query, description = "Optional comma-separated relations (`user` = full user per session)."),
    ),
    responses(
        (status = 200, description = "Returns active sessions for the specified user. `X-Total-Count` is the total before paging.", body = [SessionBody]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    req: HttpRequest,
    svc: Data<SessionServiceHandle>,
    path: Path<UserIdPath>,
    query: Query<SessionsPageQuery>,
) -> Result<HttpResponse, AppError> {
    let SessionsPageQuery { page, expand } = query.into_inner();
    let page = page
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let expand_user = expand_includes_user(&expand);
    let q_link = page.clone();
    let cur_page = page.page.unwrap_or(0);
    let page_size = page.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let sessions = svc.get_sessions_by_user_id(&path.user_id).await?;
    let lq = page.as_list_query();
    let (sessions_page, total) = ListQuery::paginate_vec(sessions, &lq);
    let sessions_page: Vec<SessionBody> = sessions_page
        .into_iter()
        .map(|s| SessionBody::from_session(s, expand_user))
        .collect();
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .insert_header((
            header::LINK,
            crate::request_link::list_link_header(
                &req,
                |p| q_link.query_string_for_page(p),
                cur_page,
                page_size,
                total,
            ),
        ))
        .json(sessions_page))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/sessions/{id}",
    params(
        ("user_id" = String, Path, description = "User identifier"),
        ("id" = String, Path, description = "Session identifier"),
        ("expand" = Option<String>, Query, description = "Optional `user` to embed full user."),
    ),
    responses(
        (status = 200, description = "Returns a session for the specified user", body = SessionBody),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    expand: Query<ExpandQuery>,
) -> Result<HttpResponse, AppError> {
    let session = svc.get_session_for_user(&path.id, &path.user_id).await?;
    Ok(HttpResponse::Ok().json(SessionBody::from_session(
        session,
        expand_includes_user(&expand.expand),
    )))
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
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
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
    actor: ReqData<User>,
    path: Path<UserSessionPath>,
) -> Result<HttpResponse, AppError> {
    let actor = actor.into_inner();
    let deleted = svc.delete_session_for_user(&path.id, &path.user_id).await?;
    crate::audit!(
        "audit.session.revoked",
        session_id = tracing::field::display(&deleted.id),
        user_id = tracing::field::display(&deleted.user.id),
        actor_user_id = tracing::field::display(&actor.id)
        ; "session revoked"
    );
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
