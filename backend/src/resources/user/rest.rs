use actix_web::{
    HttpResponse, Scope, delete, get, post,
    web::{self, Data, Json, Path, ReqData},
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;

use super::{Model, Role, User, session};
use crate::auth::middleware::RequireAdmin;
use crate::database::Database;
use crate::error::AppError;

pub fn scope() -> Scope {
    web::scope("/users")
        .service(get_user)
        .service(session::rest::get_sessions_for_current_user)
        .service(session::rest::get_session_for_current_user)
        .service(session::rest::delete_session_for_current_user)
        .service(
            web::scope("")
                .wrap(RequireAdmin::default())
                .service(create_user)
                .service(delete_user)
                .service(get_user_by_id)
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
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to load user session", body = ErrorResponse)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/me")]
async fn get_user(user: ReqData<User>) -> HttpResponse {
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
        (status = 400, description = "Invalid user identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch user", body = ErrorResponse)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_user_by_id(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_user(&id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "Returns list of all users", body = [User]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to list users", body = ErrorResponse)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_users(db: Data<Database>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_users().await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Creates a new user", body = User),
        (status = 400, description = "Invalid request payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 409, description = "User with that email already exists", body = ErrorResponse),
        (status = 500, description = "Failed to create user", body = ErrorResponse)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_user(
    db: Data<Database>,
    payload: Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_user(payload.into_inner().into_user()).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User identifier")
    ),
    responses(
        (status = 200, description = "Deletes the provided user", body = User),
        (status = 400, description = "Invalid user identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to delete user", body = ErrorResponse)
    ),
    tag = "Users",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_user(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_user(&id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

impl CreateUserRequest {
    pub fn into_user(self) -> User {
        User {
            id: String::new(),
            email: self.email,
            role: self.role,
            read: self.read,
            write: self.write,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        }
    }
}
