use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::{ErrorResponse, ProblemDetails};
use crate::accept::accepts_worship_player_json;
use crate::error::AppError;
use crate::http_cache::{if_none_match_matches, weak_etag_json};
use crate::resources::User;
use crate::resources::setlist::CreateSetlist;
use crate::resources::setlist::PatchSetlist;
#[allow(unused_imports)]
use crate::resources::setlist::Setlist;
use crate::resources::setlist::SetlistServiceHandle;
#[allow(unused_imports)]
use crate::resources::song::Song;
use crate::resources::team::UserPermissions;
use shared::api::ListQuery;
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/setlists")
        .service(get_setlists)
        .service(get_setlist)
        .service(get_setlist_songs)
        .service(get_setlist_player)
        .service(create_setlist)
        .service(update_setlist)
        .service(patch_setlist)
        .service(delete_setlist)
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists",
    params(
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Defaults to 0."),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50."),
        ("q" = Option<String>, Query, description = "Full-text search query (title); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all setlists. `X-Total-Count` header contains the total number of matching setlists.", body = [Setlist]),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
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
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let perms = UserPermissions::new(&user, &svc.teams);
    let q_ref = query.q.clone();
    let setlists = svc.list_setlists_for_user(&perms, query).await?;
    let total = svc
        .count_setlists_for_user(&perms, q_ref.as_deref())
        .await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(setlists))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return a single setlist (weak `ETag`; `If-None-Match` supported)", body = Setlist),
        (status = 304, description = "Not modified"),
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
    req: HttpRequest,
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    let setlist = svc.get_setlist_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&setlist).map_err(|e| AppError::Internal(e.to_string()))?;
    if if_none_match_matches(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .finish());
    }
    Ok(HttpResponse::Ok()
        .insert_header((header::ETAG, etag))
        .json(setlist))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/player",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a setlist", body = Player),
        (status = 400, description = "Invalid setlist identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Setlist not found", body = ProblemDetails),
        (status = 406, description = "No supported representation in Accept header", body = ProblemDetails),
        (status = 500, description = "Failed to fetch setlist player data", body = ProblemDetails)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_setlist_player(
    req: HttpRequest,
    svc: Data<SetlistServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    if !accepts_worship_player_json(&req) {
        return Err(AppError::not_acceptable(
            "supported Accept values include application/json, application/vnd.worship.player+json, and */*",
        ));
    }
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.setlist_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/songs",
    params(
        ("id" = String, Path, description = "Setlist identifier"),
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Omit with `page_size` for full list."),
        ("page_size" = Option<u32>, Query, description = "Items per page (1–500). Omit with `page` for full list."),
        ("q" = Option<String>, Query, description = "Reserved; not used for this sub-resource.")
    ),
    responses(
        (status = 200, description = "Return the songs for a setlist. `X-Total-Count` is the total before paging.", body = [Song]),
        (status = 400, description = "Invalid setlist identifier or pagination", body = ErrorResponse),
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
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let perms = UserPermissions::new(&user, &svc.teams);
    let (songs, total) = svc.setlist_songs_for_user(&perms, &id, query).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(songs))
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
        (status = 204, description = "Setlist deleted"),
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
    svc.delete_setlist_for_user(&perms, &id).await?;
    Ok(HttpResponse::NoContent().finish())
}
