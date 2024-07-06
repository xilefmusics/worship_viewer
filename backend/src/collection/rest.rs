use super::{Collection, Model};
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::user::Model as UserModel;

use fancy_surreal::Client;

use actix_web::{
    delete, get, post, put, web::Data, web::Json, web::Path, HttpRequest, HttpResponse,
};

#[get("/api/collections")]
pub async fn get(req: HttpRequest, db: Data<Client<'_>>) -> Result<HttpResponse, AppError> {
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

#[get("/api/collections/{id}")]
pub async fn get_id(
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

#[put("/api/collections")]
pub async fn put(
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created().json(
        Model::put(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            collections.into_inner(),
        )
        .await?,
    ))
}

#[post("/api/collections")]
pub async fn post(
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created().json(
        Model::create(
            db.clone(),
            &parse_user_header(&req)?,
            collections.into_inner(),
        )
        .await?,
    ))
}

#[delete("/api/collections")]
pub async fn delete(
    req: HttpRequest,
    collections: Json<Vec<Collection>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::NoContent().json(
        Model::delete(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            collections.into_inner(),
        )
        .await?,
    ))
}
