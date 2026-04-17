use actix_web::{
    HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::setlist::CreateSetlist;
use crate::resources::setlist::PatchSetlist;
#[allow(unused_imports)]
use crate::resources::setlist::Setlist;
use crate::resources::setlist::SetlistServiceHandle;
#[allow(unused_imports)]
use crate::resources::song::{Format, QueryParams, Song};
use crate::resources::team::UserPermissions;
use crate::settings::PrinterConfig;
use shared::api::ListQuery;
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/setlists")
        .service(get_setlists)
        .service(get_setlist)
        .service(get_setlist_songs)
        .service(get_setlist_player)
        .service(get_setlist_export)
        .service(create_setlist)
        .service(update_setlist)
        .service(patch_setlist)
        .service(delete_setlist)
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists",
    params(
        ("page" = Option<u32>, Query, description = "Optional page index (zero-based)"),
        ("page_size" = Option<u32>, Query, description = "Optional page size (number of items per page)"),
        ("q" = Option<String>, Query, description = "Full-text search query (title); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all setlists", body = [Setlist]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlists", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_setlists(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.list_setlists_for_user(&perms, query.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return a single setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_setlist(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.get_setlist_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/player",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a setlist", body = Player),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist player data", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_setlist_player(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.setlist_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/export",
    params(
        ("id" = String, Path, description = "Setlist identifier"),
        ("format" = Format, Query, description = "Optional export format: zip, wp, cp, pdf (defaults to wp)")
    ),
    responses(
        (
            status = 200,
            description = "Download exported setlist file",
            body = Vec<u8>,
            content_type = "application/octet-stream"
        ),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to export setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/export")]
async fn get_setlist_export(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    printer: Data<PrinterConfig>,
    id: Path<String>,
    query: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(svc
        .export_setlist_for_user(&perms, &id, query.into_inner().format, &printer)
        .await?
        .into())
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/songs",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return the songs for a setlist", body = [Song]),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist songs", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/songs")]
async fn get_setlist_songs(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.setlist_songs_for_user(&perms, &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/setlists",
    request_body = CreateSetlist,
    responses(
        (status = 201, description = "Create a new setlist", body = Setlist),
        (status = 400, description = "Invalid setlist payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_setlist(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    payload: Json<CreateSetlist>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Created().json(
        svc.create_setlist_for_user(&perms, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    request_body = CreateSetlist,
    responses(
        (status = 200, description = "Update an existing setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to update setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_setlist(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateSetlist>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.update_setlist_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    request_body = PatchSetlist,
    responses(
        (status = 200, description = "Partially update an existing setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier or payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to patch setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[patch("/{id}")]
async fn patch_setlist(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<PatchSetlist>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.patch_setlist_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Delete a setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_setlist(
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.delete_setlist_for_user(&perms, &id).await?))
}
