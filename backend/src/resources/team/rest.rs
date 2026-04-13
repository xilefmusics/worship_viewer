use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, ReqData},
};

use super::model::TeamModel;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::{User, UserRole};
#[allow(unused_imports)]
use shared::team::Team;
use shared::team::{CreateTeam, UpdateTeam};

pub fn scope() -> Scope {
    web::scope("/teams")
        .service(get_teams)
        .service(get_team)
        .service(create_team)
        .service(update_team)
        .service(delete_team)
}

#[utoipa::path(
    get,
    path = "/api/v1/teams",
    responses(
        (status = 200, description = "Teams readable by the current user; platform admins receive all teams (except internal public)", body = [Team]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to list teams", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_teams(db: Data<Database>, user: ReqData<User>) -> Result<HttpResponse, AppError> {
    let app_admin = user.role == UserRole::Admin;
    Ok(HttpResponse::Ok().json(db.get_teams(&user.id, app_admin).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{id}",
    params(
        ("id" = String, Path, description = "Team identifier")
    ),
    responses(
        (status = 200, description = "Team details; platform admins may read any team except internal public", body = Team),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Team not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch team", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_team(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let app_admin = user.role == UserRole::Admin;
    Ok(HttpResponse::Ok().json(db.get_team(&user.id, &id, app_admin).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/teams",
    request_body = CreateTeam,
    responses(
        (status = 201, description = "Shared team created", body = Team),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create team", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_team(
    db: Data<Database>,
    user: ReqData<User>,
    payload: Json<CreateTeam>,
) -> Result<HttpResponse, AppError> {
    let team = db
        .create_shared_team(&user.id, payload.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(team))
}

#[utoipa::path(
    put,
    path = "/api/v1/teams/{id}",
    params(
        ("id" = String, Path, description = "Team identifier")
    ),
    request_body = UpdateTeam,
    responses(
        (status = 200, description = "Team updated", body = Team),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient team role", body = ErrorResponse),
        (status = 404, description = "Team not found", body = ErrorResponse),
        (status = 409, description = "Sole admin cannot remove all admins", body = ErrorResponse),
        (status = 500, description = "Failed to update team", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_team(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<UpdateTeam>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.update_team(&user.id, &id, payload.into_inner()).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/teams/{id}",
    params(
        ("id" = String, Path, description = "Team identifier")
    ),
    responses(
        (status = 200, description = "Shared team deleted", body = Team),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Cannot delete personal team or insufficient role", body = ErrorResponse),
        (status = 404, description = "Team not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete team", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_team(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_team(&user.id, &id).await?))
}
