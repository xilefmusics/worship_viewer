#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use shared::api::ListQuery;
use actix_web::{
    HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};
use actix_web::http::header;
#[allow(unused_imports)]
use shared::team::Team;
use shared::team::{CreateTeam, PatchTeam, UpdateTeam};

use super::invitation;
use super::service::TeamServiceHandle;

pub fn scope() -> Scope {
    web::scope("/teams")
        .service(invitation::rest::team_invitations_scope())
        .service(get_teams)
        .service(get_team)
        .service(create_team)
        .service(update_team)
        .service(patch_team)
        .service(delete_team)
}

#[utoipa::path(
    get,
    path = "/api/v1/teams",
    params(
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Omit with `page_size` for full list (see nested pagination)."),
        ("page_size" = Option<u32>, Query, description = "Items per page (1–500). Omit with `page` for full list."),
        ("q" = Option<String>, Query, description = "Reserved; not used for teams.")
    ),
    responses(
        (status = 200, description = "Teams readable by the current user; platform admins receive all teams (except internal public). `X-Total-Count` is the total before paging.", body = [Team]),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
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
async fn get_teams(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let teams = svc.list_teams_for_user(&user).await?;
    let total = teams.len() as u64;
    let (page, _) = ListQuery::paginate_nested_vec(teams, &query);
    Ok(HttpResponse::Ok()
        .insert_header((header::HeaderName::from_static("x-total-count"), total.to_string()))
        .json(page))
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
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_team_for_user(&user, &id).await?))
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
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    payload: Json<CreateTeam>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(
        svc.create_shared_team_for_user(&user, payload.into_inner())
            .await?,
    ))
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
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<UpdateTeam>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.update_team_for_user(&user, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/teams/{id}",
    params(
        ("id" = String, Path, description = "Team identifier")
    ),
    request_body = PatchTeam,
    responses(
        (status = 200, description = "Team partially updated", body = Team),
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
#[patch("/{id}")]
async fn patch_team(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<PatchTeam>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.patch_team_for_user(&user, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/teams/{id}",
    params(
        ("id" = String, Path, description = "Team identifier")
    ),
    responses(
        (status = 204, description = "Team deleted"),
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
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    svc.delete_team_for_user(&user, &id).await?;
    Ok(HttpResponse::NoContent().finish())
}
