use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::http_cache::{if_none_match_matches, weak_etag_json};
use crate::resources::User;
use crate::resources::song::CreateSong;
use crate::resources::song::PatchSong;
#[allow(unused_imports)]
use crate::resources::song::Song;
use crate::resources::song::service::SongServiceHandle;
use crate::resources::team::UserPermissions;
use shared::api::ListQuery;
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
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Defaults to 0."),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50."),
        ("q" = Option<String>, Query, description = "Full-text search query (titles, artists, line lyrics); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all songs. `X-Total-Count` header contains the total number of matching songs.", body = [Song]),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch songs", body = ErrorResponse)
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
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let perms = UserPermissions::new(&user, &svc.teams);
    let q_ref = query.q.clone();
    let songs = svc.list_songs_for_user(&perms, query).await?;
    let total = svc.count_songs_for_user(&perms, q_ref.as_deref()).await?;
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
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
        (status = 200, description = "Return player metadata for a song", body = Player),
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch song player data", body = ErrorResponse)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_song_player(
    svc: Data<SongServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.song_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/songs",
    request_body = CreateSong,
    responses(
        (status = 201, description = "Create a new song. If the user has no default collection yet, the server may create a system \"Default\" collection and set it as `user.default_collection` (BLC-SONG-010).", body = Song),
        (status = 400, description = "Invalid song payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Created().json(
        svc.create_song_for_user(&perms, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = CreateSong,
    responses(
        (status = 200, description = "Update an existing song, or upsert: if the id does not exist, creates the song (same as POST) on the caller's writable context (BLC upsert tests).", body = Song),
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to update song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.update_song_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
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
        (status = 400, description = "Invalid song identifier or payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to patch song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to get like status for a song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to update like status for a song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to update like status for a song", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
    svc.set_song_like_status_for_user(&perms, &id, false).await?;
    Ok(HttpResponse::NoContent().finish())
}
