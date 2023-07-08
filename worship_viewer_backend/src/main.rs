mod database;
mod error;
mod types;

use actix_files::NamedFile;
use actix_web::{
    get, middleware::Logger, post, web::Data, web::Json, web::Path, App, HttpRequest, HttpResponse,
    HttpServer,
};
use database::Database;
use env_logger::Env;
use error::AppError;
use std::path::PathBuf;
use types::{Blob, Collection, Group, Song, UserGroupsId};

pub fn parse_user_header(req: HttpRequest) -> Result<String, AppError> {
    Ok(req
        .headers()
        .get("X-Remote-User")
        .ok_or(AppError::Unauthorized("no X-Remote-User given".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("no X-Remote-User given".into()))?
        .into())
}

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

#[get("/api/groups")]
pub async fn get_groups(db: Data<Database>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }

    Ok(HttpResponse::Ok().json(db.get_groups().await?))
}

#[get("/api/groups/{id}")]
pub async fn get_groups_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }

    Ok(HttpResponse::Ok().json(db.get_group(&id).await?))
}

#[post("/api/groups")]
pub async fn post_groups(
    db: Data<Database>,
    req: HttpRequest,
    groups: Json<Vec<Group>>,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }
    Ok(HttpResponse::Ok().json(db.add_groups(&groups).await?))
}

#[get("/api/users")]
pub async fn get_users(db: Data<Database>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }

    Ok(HttpResponse::Ok().json(db.get_users().await?))
}

#[get("/api/users/{id}")]
pub async fn get_users_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }

    Ok(HttpResponse::Ok().json(db.get_user(&id).await?))
}

#[post("/api/users")]
pub async fn post_users(
    db: Data<Database>,
    req: HttpRequest,
    users: Json<Vec<UserGroupsId>>,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }
    Ok(HttpResponse::Ok().json(db.add_users(&users).await?))
}

#[get("/api/blobs/{id}")]
pub async fn get_blobs_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<NamedFile, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    if !db.check_blob(&id, &username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have the rights to access the blob".into(),
        ));
    }
    let root_path = PathBuf::from(std::env::var("BLOB_DIR").unwrap_or("blobs".into()));
    let file_path = PathBuf::from(format!("{}.png", id));
    let path = root_path.join(file_path);
    Ok(NamedFile::open(path).map_err(|err| AppError::Filesystem(format!("{}", err)))?)
}

#[get("/api/blobs/metadata")]
pub async fn get_blobs_metadata(
    db: Data<Database>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_blobs(&username).await?))
}

#[get("/api/blobs/metadata/{id}")]
pub async fn get_blobs_metadata_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_blob(&username, &id).await?))
}

#[post("/api/blobs/metadata")]
pub async fn post_blobs_metadata(
    db: Data<Database>,
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }
    Ok(HttpResponse::Ok().json(db.add_blobs(&blobs).await?))
}

#[get("/api/songs")]
pub async fn get_songs(db: Data<Database>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_songs(&username).await?))
}

#[get("/api/songs/{id}")]
pub async fn get_songs_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_song(&username, &id).await?))
}

#[post("/api/songs")]
pub async fn post_songs(
    db: Data<Database>,
    req: HttpRequest,
    songs: Json<Vec<Song>>,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }
    Ok(HttpResponse::Ok().json(db.add_songs(&songs).await?))
}

#[post("/api/collections")]
pub async fn post_collections(
    db: Data<Database>,
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    if !db.check_user_admin(&username).await? {
        return Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ));
    }
    Ok(HttpResponse::Ok().json(db.add_collections(&collections).await?))
}

#[get("/api/collections")]
pub async fn get_collections(
    db: Data<Database>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_collections(&username).await?))
}

#[get("/api/collections/{id}")]
pub async fn get_collections_id(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    Ok(HttpResponse::Ok().json(db.get_collection(&username, &id).await?))
}

#[get("/api/player/{id}")]
pub async fn get_player(
    db: Data<Database>,
    req: HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let username = parse_user_header(req)?;
    if id.starts_with("song:") {
        Ok(HttpResponse::Ok().json(db.get_song(&username, &id).await?.to_player()?))
    } else if id.starts_with("collection:") {
        Ok(HttpResponse::Ok().json(
            db.get_collection_fetched_songs(&username, &id)
                .await?
                .to_player()?,
        ))
    } else {
        Err(AppError::NotFound(format!(
            "Can not create player for id {}",
            id
        )))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let port = std::env::var("PORT")
        .unwrap_or("8082".into())
        .parse::<u16>()
        .unwrap_or(8082);

    let db_host = std::env::var("DB_HOST").unwrap_or("localhost".into());

    let db_port = std::env::var("DB_PORT")
        .unwrap_or("8000".into())
        .parse::<u16>()
        .unwrap_or(8000);

    let db_user = std::env::var("DB_USER").unwrap_or("root".into());

    let db_password = std::env::var("DB_PASSWORD").unwrap_or("root".into());

    let db_namespace = std::env::var("DB_NAMESPACE").unwrap_or("test".into());

    let db_database = std::env::var("DB_DATABASE").unwrap_or("test".into());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Database::new(
                &db_host,
                db_port,
                &db_user,
                &db_password,
                &db_namespace,
                &db_database,
            )))
            .service(get_groups)
            .service(get_groups_id)
            .service(post_groups)
            .service(get_users)
            .service(get_users_id)
            .service(post_users)
            .service(get_blobs_metadata)
            .service(get_blobs_metadata_id)
            .service(post_blobs_metadata)
            .service(get_blobs_id)
            .service(get_songs)
            .service(get_songs_id)
            .service(post_songs)
            .service(get_collections)
            .service(get_collections_id)
            .service(post_collections)
            .service(get_player)
            .service(index)
            .service(static_files)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
