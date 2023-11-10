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
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "group",
            groups
                .clone()
                .into_iter()
                .map(|group| GroupDatabase::try_from(group))
                .collect::<Result<Vec<GroupDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|group| group.into())
        .collect::<Vec<Group>>(),
    ))
}

#[post("/api/users")]
pub async fn users(
    req: HttpRequest,
    users: Json<Vec<User>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "user",
            users
                .clone()
                .into_iter()
                .map(|user| UserDatabase::try_from(user))
                .collect::<Result<Vec<UserDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|user| user.into())
        .collect::<Vec<User>>(),
    ))
}

#[post("/api/blobs/metadata")]
pub async fn blobs_metadata(
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "blob",
            blobs
                .clone()
                .into_iter()
                .map(|blob| BlobDatabase::try_from(blob))
                .collect::<Result<Vec<BlobDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|blob| blob.into())
        .collect::<Vec<Blob>>(),
    ))
}

#[post("/api/songs")]
pub async fn songs(
    req: HttpRequest,
    songs: Json<Vec<Song>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "song",
            songs
                .clone()
                .into_iter()
                .map(|song| SongDatabase::try_from(song))
                .collect::<Result<Vec<SongDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|song| song.into())
        .collect::<Vec<Song>>(),
    ))
}

#[post("/api/collections")]
pub async fn collections(
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "collection",
            collections
                .clone()
                .into_iter()
                .map(|collection| CollectionDatabase::try_from(collection))
                .collect::<Result<Vec<CollectionDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|collection| collection.into())
        .collect::<Vec<Collection>>(),
    ))
}
