use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, ReqData},
};

use super::Model;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::song::CreateSong;
#[allow(unused_imports)]
use crate::resources::song::Song;
use shared::like::LikeStatus;
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/songs")
        .service(get_songs)
        .service(get_song)
        .service(get_song_player)
        .service(create_song)
        .service(update_song)
        .service(delete_song)
        .service(get_song_like_status)
        .service(update_song_like_status)
}

#[utoipa::path(
    get,
    path = "/api/v1/songs",
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
async fn get_songs(db: Data<Database>, user: ReqData<User>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_songs(user.read()).await?))
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
    Ok(HttpResponse::Ok().json(db.get_song(user.read(), &id).await?))
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
    Ok(HttpResponse::Ok().json(Player::from(db.get_song(user.read(), &id).await?)))
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
    Ok(HttpResponse::Created().json(db.create_song(&user.id, payload.into_inner()).await?))
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
        db.update_song(user.write(), &user.id, &id, payload.into_inner())
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
    Ok(HttpResponse::Ok().json(db.delete_song(user.write(), &id).await?))
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
    let liked = db.get_song_like(user.read(), &user.id, &id).await?;

    Ok(HttpResponse::Ok().json(LikeStatus { liked }))
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
    let LikeStatus { liked } = payload.into_inner();
    let liked = db.set_song_like(user.read(), &user.id, &id, liked).await?;

    Ok(HttpResponse::Ok().json(LikeStatus { liked }))
}
