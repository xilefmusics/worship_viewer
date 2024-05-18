use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{
    Blob, BlobDatabase, Collection, CollectionDatabase, Group, GroupDatabase, Song, SongDatabase,
    User, UserDatabase,
};
use crate::AppError;

use actix_web::{post, web::Data, web::Json, HttpRequest, HttpResponse};

#[post("/api/groups")]
pub async fn groups(
    req: HttpRequest,
    groups: Json<Vec<Group>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(GroupDatabase::create(&db, groups.into_inner()).await?))
}

#[post("/api/users")]
pub async fn users(
    req: HttpRequest,
    users: Json<Vec<User>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(UserDatabase::create(&db, users.into_inner()).await?))
}

#[post("/api/blobs/metadata")]
pub async fn blobs_metadata(
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(BlobDatabase::create(&db, blobs.into_inner()).await?))
}

#[post("/api/songs")]
pub async fn songs(
    req: HttpRequest,
    songs: Json<Vec<Song>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(SongDatabase::create(&db, songs.into_inner()).await?))
}

#[post("/api/collections")]
pub async fn collections(
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(CollectionDatabase::create(&db, collections.into_inner()).await?))
}
