use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{
    Blob, BlobDatabase, Collection, CollectionDatabase, Group, GroupDatabase, Song, SongDatabase,
    User, UserDatabase,
};
use crate::AppError;

use actix_web::{get, web::Data, web::Path, HttpRequest, HttpResponse};

#[get("/api/groups/{id:group.*}")]
pub async fn groups_id(
    req: HttpRequest,
    db: Data<Database>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select::<GroupDatabase>("group", None, None, None, Some(&id.into_inner()))
            .await?
            .into_iter()
            .map(|group| group.into())
            .collect::<Vec<Group>>(),
    ))
}

#[get("/api/groups")]
pub async fn groups(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select::<GroupDatabase>("group", None, None, None, None)
            .await?
            .into_iter()
            .map(|group| group.into())
            .collect::<Vec<Group>>(),
    ))
}

#[get("/api/users/{id:user.*}")]
pub async fn users_id(
    req: HttpRequest,
    db: Data<Database>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select::<UserDatabase>("user", None, None, None, Some(&id.into_inner()))
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<User>>(),
    ))
}

#[get("/api/users")]
pub async fn users(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select::<UserDatabase>("user", None, None, None, None)
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<User>>(),
    ))
}

#[get("/api/blobs/metadata/{id:blob.*}")]
pub async fn blobs_metadata_id(
    req: HttpRequest,
    db: Data<Database>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<BlobDatabase>("blob", None, None, Some(user), Some(&id.into_inner()))
            .await?
            .into_iter()
            .map(|blob| blob.into())
            .collect::<Vec<Blob>>(),
    ))
}

#[get("/api/blobs/metadata")]
pub async fn blobs_metadata(
    req: HttpRequest,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<BlobDatabase>("blob", None, None, Some(user), None)
            .await?
            .into_iter()
            .map(|blob| blob.into())
            .collect::<Vec<Blob>>(),
    ))
}

#[get("/api/songs/{id:song.*}")]
pub async fn songs_id(
    req: HttpRequest,
    db: Data<Database>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<SongDatabase>("song", None, None, Some(user), Some(&id.into_inner()))
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>(),
    ))
}

#[get("/api/songs")]
pub async fn songs(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<SongDatabase>("song", None, None, Some(user), None)
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>(),
    ))
}

#[get("/api/collections/{id:collection.*}")]
pub async fn collections_id(
    req: HttpRequest,
    db: Data<Database>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<CollectionDatabase>(
            "collection",
            None,
            None,
            Some(user),
            Some(&id.into_inner()),
        )
        .await?
        .into_iter()
        .map(|collection| collection.into())
        .collect::<Vec<Collection>>(),
    ))
}

#[get("/api/collections")]
pub async fn collections(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<CollectionDatabase>("collection", None, None, Some(user), None)
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect::<Vec<Collection>>(),
    ))
}
