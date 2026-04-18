use super::{CreateUserRequest, User, session};
use crate::auth::middleware::RequireAdmin;
#[allow(unused_imports)]
use crate::docs::ProblemDetails;
use crate::error::AppError;
use crate::resources::user::service::UserServiceHandle;
use actix_web::http::header;
use actix_web::{
    HttpResponse, Scope, delete, get, post,
    web::{self, Data, Json, Path, Query, ReqData},
};
use shared::api::ListQuery;

pub fn scope() -> Scope {
    web::scope("/users")
        .service(get_users_me)
        .service(session::rest::get_sessions_for_current_user)
        .service(session::rest::get_session_for_current_user)
        .service(session::rest::delete_session_for_current_user)
        .service(
            web::scope("")
                .wrap(RequireAdmin)
                .service(create_user)
                .service(delete_user)
                .service(get_user)
                .service(get_users)
                .service(session::rest::get_sessions_for_user)
                .service(session::rest::get_session_for_user)
                .service(session::rest::create_session_for_user)
                .service(session::rest::delete_session_for_user),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "Returns the currently authenticated user", body = User),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 500, description = "Failed to load user session", body = ProblemDetails)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/me")]
async fn get_users_me(user: ReqData<User>) -> HttpResponse {
    HttpResponse::Ok().json(user.into_inner())
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 200, description = "Returns the user matching the provided id", body = User),
        (status = 400, description = "Invalid user identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 403, description = "Admin role required", body = ProblemDetails),
        (status = 500, description = "Failed to fetch user", body = ProblemDetails)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_user(
    svc: Data<UserServiceHandle>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_user(&id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Defaults to 0. See `docs/business-logic-constraints/list-pagination.md` (Track A: `X-Total-Count` is pre-pagination total; last page when `items.len() < page_size` or empty)."),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50."),
        ("q" = Option<String>, Query, description = "Optional case-insensitive email substring filter. Whitespace-only is treated as absent.")
    ),
    responses(
        (status = 200, description = "Returns list of all users. `X-Total-Count` header contains the total matching user count (before pagination).", body = [User]),
        (status = 400, description = "Invalid pagination parameters", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 403, description = "Admin role required", body = ProblemDetails),
        (status = 500, description = "Failed to list users", body = ProblemDetails)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_users(
    svc: Data<UserServiceHandle>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let users = svc.get_users(query.clone()).await?;
    let total = svc.count_users(query).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(users))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Creates a new user", body = User),
        (status = 400, description = "Invalid request payload", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 403, description = "Admin role required", body = ProblemDetails),
        (status = 409, description = "User with that email already exists", body = ProblemDetails),
        (status = 500, description = "Failed to create user", body = ProblemDetails)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_user(
    svc: Data<UserServiceHandle>,
    payload: Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(svc.create_user_from_request(payload.into_inner()).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 400, description = "Invalid user identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 403, description = "Admin role required", body = ProblemDetails),
        (status = 500, description = "Failed to delete user", body = ProblemDetails)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_user(
    svc: Data<UserServiceHandle>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    svc.delete_user(&id).await?;
    Ok(HttpResponse::NoContent().finish())
}
