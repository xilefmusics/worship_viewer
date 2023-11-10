use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{
    Blob, BlobDatabase, Collection, CollectionDatabase, Group, GroupDatabase, Song, SongDatabase,
    User, UserDatabase,
};
use crate::AppError;

use actix_files::NamedFile;
use actix_web::{get, web::Data, web::Path, HttpRequest, HttpResponse};
use std::path::PathBuf;

#[get("/")]
pub async fn index() -> Result<NamedFile, AppError> {
    let root_path = PathBuf::from(std::env::var("STATIC_DIR").unwrap_or("static".into()));
    let file_path = PathBuf::from("index.html");
    NamedFile::open(root_path.join(file_path)).map_err(|err| AppError::NotFound(err.to_string()))
}

#[get("/{path}")]
pub async fn static_files(path: Path<String>) -> Result<NamedFile, AppError> {
    let root_path = PathBuf::from(std::env::var("STATIC_DIR").unwrap_or("static".into()));
    let file_path = PathBuf::from(path.into_inner());
    let path = root_path.join(file_path);
    if path.extension().is_some() {
        NamedFile::open(path).map_err(|err| AppError::NotFound(err.to_string()))
    } else {
        NamedFile::open(root_path.join("index.html"))
            .map_err(|err| AppError::NotFound(err.to_string()))
    }
}

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

#[get("/api/blobs/{id:blob.*}")]
pub async fn blobs_id(
    db: Data<Database>,
    req: HttpRequest,
    id: Path<String>,
) -> Result<NamedFile, AppError> {
    let user = parse_user_header(req)?;
    Ok(NamedFile::open(
        PathBuf::from(std::env::var("BLOB_DIR").unwrap_or("blobs".into())).join(PathBuf::from(
            dbg!(db
                .select::<BlobDatabase>("blob", None, None, Some(&user), Some(&id.into_inner()))
                .await?
                .into_iter()
                .map(|blob| blob.into())
                .collect::<Vec<Blob>>()
                .get(0)
                .ok_or(AppError::NotFound("blob not found".into()))?
                .file_name()?),
        )),
    )
    .map_err(|err| AppError::Filesystem(format!("{}", err)))?)
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

#[get("/api/player/{id:song.*}")]
pub async fn player_id_song(
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
            .collect::<Vec<Song>>()
            .get(0)
            .ok_or(AppError::NotFound("song not found".into()))?
            .clone()
            .to_player_data()
            .map_err(|error| AppError::Other(error))?,
    ))
}
