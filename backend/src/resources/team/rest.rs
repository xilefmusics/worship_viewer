#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, ReqData},
};
#[allow(unused_imports)]
use shared::team::Team;
#[allow(unused_imports)]
use shared::team::TeamInvitation;
use shared::team::{CreateTeam, UpdateTeam};

use super::service::TeamServiceHandle;

pub fn scope() -> Scope {
    web::scope("/teams")
        .service(team_invitations_scope())
        .service(get_teams)
        .service(get_team)
        .service(create_team)
        .service(update_team)
        .service(delete_team)
}

pub fn invitations_accept_scope() -> Scope {
    web::scope("/invitations").service(accept_team_invitation)
}

fn team_invitations_scope() -> Scope {
    web::scope("/{team_id}/invitations")
        .service(create_team_invitation)
        .service(list_team_invitations)
        .service(get_team_invitation)
        .service(delete_team_invitation)
}

#[utoipa::path(
    post,
    path = "/api/v1/teams/{team_id}/invitations",
    params(
        ("team_id" = String, Path, description = "Shared team identifier")
    ),
    responses(
        (status = 201, description = "Invitation created", body = TeamInvitation),
        (status = 400, description = "Team is not shared", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Not a team admin", body = ErrorResponse),
        (status = 404, description = "Team not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_team_invitation(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    team_id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(
        svc.create_invitation_for_user(&user, team_id.as_str()).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/invitations",
    params(
        ("team_id" = String, Path, description = "Shared team identifier")
    ),
    responses(
        (status = 200, description = "Invitations for the team", body = [TeamInvitation]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Not a team admin", body = ErrorResponse),
        (status = 404, description = "Team not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn list_team_invitations(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    team_id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.list_invitations_for_user(&user, team_id.as_str()).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/invitations/{invitation_id}",
    params(
        ("team_id" = String, Path, description = "Shared team identifier"),
        ("invitation_id" = String, Path, description = "Invitation identifier")
    ),
    responses(
        (status = 200, description = "Invitation details", body = TeamInvitation),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Not a team admin", body = ErrorResponse),
        (status = 404, description = "Team or invitation not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{invitation_id}")]
async fn get_team_invitation(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (team_id, invitation_id) = path.into_inner();
    Ok(HttpResponse::Ok().json(
        svc.get_invitation_for_user(&user, &team_id, &invitation_id).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/teams/{team_id}/invitations/{invitation_id}",
    params(
        ("team_id" = String, Path, description = "Shared team identifier"),
        ("invitation_id" = String, Path, description = "Invitation identifier")
    ),
    responses(
        (status = 204, description = "Invitation removed"),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Not a team admin", body = ErrorResponse),
        (status = 404, description = "Team or invitation not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{invitation_id}")]
async fn delete_team_invitation(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (team_id, invitation_id) = path.into_inner();
    svc.delete_invitation_for_user(&user, &team_id, &invitation_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/api/v1/invitations/{invitation_id}/accept",
    params(
        ("invitation_id" = String, Path, description = "Invitation identifier")
    ),
    responses(
        (status = 200, description = "Current user is on the team (added as guest if needed)", body = Team),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Invitation not found or not usable", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("/{invitation_id}/accept")]
async fn accept_team_invitation(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    invitation_id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.accept_invitation_for_user(&user, invitation_id.as_str()).await?,
    ))
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
async fn get_teams(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.list_teams_for_user(&user).await?))
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
        svc.create_shared_team_for_user(&user, payload.into_inner()).await?,
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
        svc.update_team_for_user(&user, &id, payload.into_inner()).await?,
    ))
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
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.delete_team_for_user(&user, &id).await?))
}
