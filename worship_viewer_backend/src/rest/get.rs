use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{
    Blob, BlobDatabase, Collection, CollectionDatabase, Group, GroupDatabase, Song, SongDatabase,
    User, UserDatabase,
};
use crate::AppError;

use actix_web::{get, web::Data, HttpRequest, HttpResponse};

#[get("/api/groups")]
pub async fn groups(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        // TODO: handle proper pagination
        db.select::<GroupDatabase>("group", None, None, None)
            .await?
            .into_iter()
            .map(|group| group.into())
            .collect::<Vec<Group>>(),
    ))
}

#[get("/api/users")]
pub async fn users(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select::<UserDatabase>("user", None, None, None)
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<User>>(),
    ))
}

#[get("/api/blobs/metadata")]
pub async fn blobs_metadata(
    req: HttpRequest,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<BlobDatabase>("blob", None, None, Some(user))
            .await?
            .into_iter()
            .map(|blob| blob.into())
            .collect::<Vec<Blob>>(),
    ))
}

#[get("/api/songs")]
pub async fn songs(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<SongDatabase>("song", None, None, Some(user))
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>(),
    ))
}

#[get("/api/collections")]
pub async fn collections(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    let user = &parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(
        db.select::<CollectionDatabase>("collection", None, None, Some(user))
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect::<Vec<Collection>>(),
    ))
}
