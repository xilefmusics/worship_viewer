use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

use crate::accept::accepts_worship_player_json;
#[allow(unused_imports)]
use crate::docs::ProblemDetails;
use crate::error::AppError;
use crate::http_cache::{if_none_match_matches, weak_etag_json};
use crate::resources::User;
use crate::resources::song::CreateSong;
use crate::resources::song::PatchSong;
#[allow(unused_imports)]
use crate::resources::song::Song;
use crate::resources::song::SongUpsertOutcome;
use crate::resources::song::service::SongServiceHandle;
use crate::resources::team::UserPermissions;
use shared::api::SongListQuery;
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/songs")
        .service(get_songs)
        .service(get_song)
        .service(get_song_player)
        .service(create_song)
        .service(update_song)
        .service(patch_song)
        .service(delete_song)
        .service(get_song_like_status)
        .service(put_song_like)
        .service(delete_song_like)
}

#[utoipa::path(
    get,
    path = "/api/v1/songs",
    params(
        ("page" = Option<u32>, Query, description = "Zero-based page index (default 0). `X-Total-Count` is the total before pagination; the last page is when `items.len() < page_size` or the list is empty (see `docs/business-logic-constraints/list-pagination.md`)."),
        ("page_size" = Option<u32>, Query, description = "Page size 1–500 (default 50)."),
        ("q" = Option<String>, Query, description = "Full-text search query (titles, artists, line lyrics); uses text_search analyzer (stemming)"),
        ("sort" = Option<String>, Query, description = "Sort: `id_desc` (default without q), `id_asc`, `title_asc`, `title_desc`, `relevance` (default with q; requires q when set explicitly)"),
        ("lang" = Option<String>, Query, description = "Filter: song must list this language in `data.languages`."),
        ("tag" = Option<String>, Query, description = "Filter: case-insensitive substring match on stringified `data.tags`.")
    ),
    responses(
        (status = 200, description = "Return all songs. `X-Total-Count` header contains the total number of matching songs.", body = [Song]),
        (status = 400, description = "Invalid pagination or filter parameters", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 500, description = "Failed to fetch songs", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_songs(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    query: Query<SongListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let songs = svc.list_songs_for_user(&perms, query.clone()).await?;
    let total = svc.count_songs_for_user(&perms, &query).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(songs))
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 200, description = "Return a single song. Response includes a weak `ETag`; send `If-None-Match` for conditional requests.", body = Song),
        (status = 304, description = "Not modified (when `If-None-Match` matches the current ETag)"),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to fetch song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_song(
    req: HttpRequest,
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let song = svc.get_song_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&song).map_err(|e| AppError::Internal(e.to_string()))?;
    if if_none_match_matches(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .finish());
    }
    Ok(HttpResponse::Ok()
        .insert_header((header::ETAG, etag))
        .json(song))
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}/player",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a song (`Content-Type: application/json`). Send `Accept: application/json`, `application/vnd.worship.player+json`, or `*/*`.", body = Player),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 406, description = "No supported representation in Accept header", body = ProblemDetails),
        (status = 500, description = "Failed to fetch song player data", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_song_player(
    req: HttpRequest,
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    if !accepts_worship_player_json(&req) {
        return Err(AppError::not_acceptable(
            "supported Accept values include application/json, application/vnd.worship.player+json, and */*",
        ));
    }
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.song_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/songs",
    request_body = CreateSong,
    responses(
        (status = 201, description = "Create a new song. If the user has no default collection yet, the server may create a system \"Default\" collection and set it as `user.default_collection` (BLC-SONG-010).", body = Song),
        (status = 400, description = "Invalid song payload", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 500, description = "Failed to create song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_song(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    payload: Json<CreateSong>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let payload = payload.into_inner();
    payload.validate().map_err(AppError::invalid_request)?;
    Ok(HttpResponse::Created().json(svc.create_song_for_user(&perms, payload).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = CreateSong,
    responses(
        (status = 200, description = "Updated an existing song.", body = Song),
        (status = 201, description = "Created the song via PUT upsert (new id). Response includes `Location: /api/v1/songs/{id}`.", body = Song),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to update song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_song(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateSong>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let payload = payload.into_inner();
    payload.validate().map_err(AppError::invalid_request)?;
    match svc.update_song_for_user(&perms, &id, payload).await? {
        SongUpsertOutcome::Created(song) => Ok(HttpResponse::Created()
            .insert_header((header::LOCATION, format!("/api/v1/songs/{}", song.id)))
            .json(song)),
        SongUpsertOutcome::Updated(song) => Ok(HttpResponse::Ok().json(song)),
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = PatchSong,
    responses(
        (status = 200, description = "Partially update an existing song", body = Song),
        (status = 400, description = "Invalid song identifier or payload", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to patch song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[patch("/{id}")]
async fn patch_song(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<PatchSong>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.patch_song_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 204, description = "Song deleted"),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to delete song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_song(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    svc.delete_song_for_user(&perms, &id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}/like",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 200, description = "Whether the current user likes this song", body = LikeStatus),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to get like status for a song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/like")]
async fn get_song_like_status(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.song_like_status_for_user(&perms, &id).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}/like",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 204, description = "Current user now likes this song"),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to update like status for a song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}/like")]
async fn put_song_like(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    svc.set_song_like_status_for_user(&perms, &id, true).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/api/v1/songs/{id}/like",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 204, description = "Current user no longer likes this song"),
        (status = 400, description = "Invalid song identifier", body = ProblemDetails),
        (status = 401, description = "Authentication required", body = ProblemDetails),
        (status = 404, description = "Song not found", body = ProblemDetails),
        (status = 500, description = "Failed to update like status for a song", body = ProblemDetails)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}/like")]
async fn delete_song_like(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    svc.set_song_like_status_for_user(&perms, &id, false)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}
