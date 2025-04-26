use super::{Blob, Model};
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::user::Model as UserModel;

use fancy_surreal::Client;

use actix_files::NamedFile;
use actix_web::{
    delete, get, post, put, web::Data, web::Json, web::Path, HttpRequest, HttpResponse,
};
use std::path::PathBuf;

#[get("/api/blobs/metadata")]
pub async fn get_metadata(
    req: HttpRequest,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Ok().json(
        Model::get(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
        )
        .await?,
    ))
}

#[get("/api/blobs/metadata/{id}")]
pub async fn get_metadata_id(
    req: HttpRequest,
    db: Data<Client<'_>>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Ok().json(
        Model::get_one(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            &id.into_inner(),
        )
        .await?,
    ))
}

#[put("/api/blobs/metadata")]
pub async fn put_metadata(
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created().json(
        Model::put(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .write,
            blobs.into_inner(),
        )
        .await?,
    ))
}

#[post("/api/blobs/metadata")]
pub async fn post_metadata(
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created()
        .json(Model::create(db.clone(), &parse_user_header(&req)?, blobs.into_inner()).await?))
}

#[delete("/api/blobs/metadata")]
pub async fn delete_metadata(
    req: HttpRequest,
    blobs: Json<Vec<Blob>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::NoContent().json(
        Model::delete(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .write,
            blobs.into_inner(),
        )
        .await?,
    ))
}

#[get("/api/blobs/{id}")]
pub async fn get_id(
    db: Data<Client<'_>>,
    req: HttpRequest,
    id: Path<String>,
) -> Result<NamedFile, AppError> {
    let db = db.into_inner();
    let metadata = Model::get_one(
        db.clone(),
        UserModel::get_or_create(db, &parse_user_header(&req)?)
            .await?
            .read,
        &id.into_inner(),
    )
    .await?;
    let file_name = metadata
        .file_name()
        .ok_or(AppError::NotFound("blob has no id".into()))?;

    let path = PathBuf::from(std::env::var("BLOB_DIR").unwrap_or("blobs".into()))
        .join(PathBuf::from(file_name));

    Ok(NamedFile::open(path).map_err(|err| AppError::Filesystem(format!("{}", err)))?)
}
