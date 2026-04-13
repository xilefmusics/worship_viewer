use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::song::CreateSong;
#[allow(unused_imports)]
use crate::resources::song::Song;
#[allow(unused_imports)]
use crate::resources::song::{Format, QueryParams};
use shared::api::ListQuery;
use shared::like::LikeStatus;
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/songs")
        .service(get_songs)
        .service(get_song)
        .service(get_song_player)
        .service(get_song_export)
        .service(create_song)
        .service(update_song)
        .service(delete_song)
        .service(get_song_like_status)
        .service(update_song_like_status)
}

#[utoipa::path(
    get,
    path = "/api/v1/songs",
    params(
        ("page" = Option<u32>, Query, description = "Optional page index (zero-based)"),
        ("page_size" = Option<u32>, Query, description = "Optional page size (number of items per page)"),
        ("q" = Option<String>, Query, description = "Full-text search query (titles, artists, line lyrics); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all songs", body = [Song]),
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
    db: Data<Database>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.list_songs_for_user(&user, query.into_inner()).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 200, description = "Return a single song", body = Song),
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_song_for_user(&user, &id).await?))
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.song_player_for_user(&user, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}/export",
    params(
        ("id" = String, Path, description = "Song identifier"),
        ("format" = super::Format, Query, description = "Optional export format: zip, wp, cp, pdf (defaults to wp)")
    ),
    responses(
        (
            status = 200,
            description = "Download exported song file",
            body = Vec<u8>,
            content_type = "application/octet-stream"
        ),
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
        (status = 500, description = "Failed to export song", body = ErrorResponse)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/export")]
async fn get_song_export(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    query: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    db.export_song_for_user(&user, &id, query.into_inner().format)
        .await
}

#[utoipa::path(
    post,
    path = "/api/v1/songs",
    request_body = CreateSong,
    responses(
        (status = 201, description = "Create a new song", body = Song),
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
    db: Data<Database>,
    user: ReqData<User>,
    payload: Json<CreateSong>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_song_for_user(&user, payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = CreateSong,
    responses(
        (status = 200, description = "Update an existing song", body = Song),
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateSong>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.update_song_for_user(&user, &id, payload.into_inner())
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
        (status = 200, description = "Delete a song", body = Song),
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_song_for_user(&user, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/songs/{id}/likes",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    responses(
        (status = 200, description = "Return like status for a song", body = LikeStatus),
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
#[get("/{id}/likes")]
async fn get_song_like_status(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.song_like_status_for_user(&user, &id).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}/likes",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = LikeStatus,
    responses(
        (status = 200, description = "Update like status for a song", body = LikeStatus),
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
#[put("/{id}/likes")]
async fn update_song_like_status(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<LikeStatus>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.set_song_like_status_for_user(&user, &id, payload.into_inner().liked)
            .await?,
    ))
}
