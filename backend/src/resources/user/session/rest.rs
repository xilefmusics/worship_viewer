use actix_web::{
    HttpResponse, delete, get, post,
    web::{Data, Path, ReqData},
};
use serde::Deserialize;

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use crate::settings::CookieConfig;

use super::service::SessionServiceHandle;

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions",
    responses(
        (status = 200, description = "Returns active sessions for the current user", body = [Session]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to list sessions for current user", body = ErrorResponse)
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
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_sessions_by_user_id(&user.id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session identifier")
    ),
    responses(
        (status = 200, description = "Returns a session for the current user", body = Session),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Session not found for current user", body = ErrorResponse),
        (status = 500, description = "Failed to fetch session", body = ErrorResponse)
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
        (status = 200, description = "Deletes a session for the current user", body = Session),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Session not found for current user", body = ErrorResponse),
        (status = 500, description = "Failed to delete session", body = ErrorResponse)
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
    Ok(HttpResponse::Ok().json(svc.delete_session_for_user(&path.id, &user.id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 201, description = "Creates a session for the specified user", body = Session),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to create session", body = ErrorResponse)
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
        ("user_id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 200, description = "Returns active sessions for the specified user", body = [Session]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to list sessions", body = ErrorResponse)
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
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_sessions_by_user_id(&path.user_id).await?))
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
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Session not found for specified user", body = ErrorResponse),
        (status = 500, description = "Failed to fetch session", body = ErrorResponse)
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
        (status = 200, description = "Deletes a session for the specified user", body = Session),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Session not found for specified user", body = ErrorResponse),
        (status = 500, description = "Failed to delete session", body = ErrorResponse)
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
    Ok(HttpResponse::Ok().json(svc.delete_session_for_user(&path.id, &path.user_id).await?))
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
