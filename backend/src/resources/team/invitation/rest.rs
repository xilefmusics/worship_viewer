#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::resources::User;
use actix_web::http::header;
use actix_web::{
    HttpResponse, Scope, delete, get, post,
    web::{self, Data, Path, Query, ReqData},
};

use shared::api::PageQuery;
#[allow(unused_imports)]
use shared::team::Team;
#[allow(unused_imports)]
use shared::team::TeamInvitation;

use super::service::InvitationServiceHandle;

pub fn team_invitations_scope() -> Scope {
    web::scope("/{team_id}/invitations")
        .service(create_team_invitation)
        .service(list_team_invitations)
        .service(get_team_invitation)
        .service(delete_team_invitation)
}

pub fn invitations_accept_scope() -> Scope {
    web::scope("/invitations").service(accept_team_invitation)
}

#[utoipa::path(
    post,
    path = "/api/v1/teams/{team_id}/invitations",
    params(
        ("team_id" = String, Path, description = "Shared team identifier")
    ),
    responses(
        (status = 201, description = "Invitation created", body = TeamInvitation),
        (status = 400, description = "Team is not shared", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Not a team admin", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Team not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_team_invitation(
    svc: Data<InvitationServiceHandle>,
    user: ReqData<User>,
    team_id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(
        svc.create_invitation_for_user(&user, team_id.as_str())
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/invitations",
    params(
        ("team_id" = String, Path, description = "Shared team identifier"),
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Omit with `page_size` for full list.", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50. Omit with `page` for full list.", minimum = 1, maximum = 500, example = 50, nullable = true),
    ),
    responses(
        (status = 200, description = "Invitations for the team. `X-Total-Count` is the total before paging.", body = [TeamInvitation]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Not a team admin", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Team not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn list_team_invitations(
    svc: Data<InvitationServiceHandle>,
    user: ReqData<User>,
    team_id: Path<String>,
    query: Query<PageQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let (invitations, total) = svc
        .list_invitations_for_user(&user, team_id.as_str(), query.as_list_query())
        .await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(invitations))
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
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Not a team admin", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Team or invitation not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{invitation_id}")]
async fn get_team_invitation(
    svc: Data<InvitationServiceHandle>,
    user: ReqData<User>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (team_id, invitation_id) = path.into_inner();
    Ok(HttpResponse::Ok().json(
        svc.get_invitation_for_user(&user, &team_id, &invitation_id)
            .await?,
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
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Not a team admin", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Team or invitation not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{invitation_id}")]
async fn delete_team_invitation(
    svc: Data<InvitationServiceHandle>,
    user: ReqData<User>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (team_id, invitation_id) = path.into_inner();
    svc.delete_invitation_for_user(&user, &team_id, &invitation_id)
        .await?;
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
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Invitation not found or not usable", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Database error", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Teams",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("/{invitation_id}/accept")]
async fn accept_team_invitation(
    svc: Data<InvitationServiceHandle>,
    user: ReqData<User>,
    invitation_id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.accept_invitation_for_user(&user, invitation_id.as_str())
            .await?,
    ))
}
