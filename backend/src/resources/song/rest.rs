use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path},
};

use super::Model;
use crate::auth::middleware::RequireAdmin;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::song::Song;

pub fn scope() -> Scope {
    web::scope("/songs")
        .service(get_songs)
        .service(get_song)
        .service(
            web::scope("")
                .wrap(RequireAdmin::default())
                .service(create_song)
                .service(update_song)
                .service(delete_song),
        )
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
async fn get_songs(db: Data<Database>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_songs().await?))
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
async fn get_song(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_song(&id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/songs",
    request_body = Song,
    responses(
        (status = 201, description = "Create a new song", body = Song),
        (status = 400, description = "Invalid song payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to create song", body = ErrorResponse)
    ),
    tag = "Songs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_song(db: Data<Database>, payload: Json<Song>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_song(payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/songs/{id}",
    params(
        ("id" = String, Path, description = "Song identifier")
    ),
    request_body = Song,
    responses(
        (status = 200, description = "Update an existing song", body = Song),
        (status = 400, description = "Invalid song identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
    id: Path<String>,
    payload: Json<Song>,
) -> Result<HttpResponse, AppError> {
    let mut song = payload.into_inner();
    song.id = Some(id.clone());
    Ok(HttpResponse::Ok().json(db.update_song(&id, song).await?))
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
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
async fn delete_song(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_song(&id).await?))
}
