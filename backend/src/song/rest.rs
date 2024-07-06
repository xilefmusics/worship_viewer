use super::{Model, QueryParams, Song};
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::user::Model as UserModel;

use fancy_surreal::Client;

use actix_web::{
    delete, get, post, put, web::Data, web::Json, web::Path, web::Query, HttpRequest, HttpResponse,
};

#[get("/api/songs")]
pub async fn get(
    req: HttpRequest,
    db: Data<Client<'_>>,
    q: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Ok().json(
        Model::get(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            &q.to_filter(),
        )
        .await?,
    ))
}

#[get("/api/songs/{id}")]
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

#[put("/api/songs")]
pub async fn put(
    req: HttpRequest,
    songs: Json<Vec<Song>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created().json(
        Model::put(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            songs.into_inner(),
        )
        .await?,
    ))
}

#[post("/api/songs")]
pub async fn post(
    req: HttpRequest,
    songs: Json<Vec<Song>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Created()
        .json(Model::create(db.clone(), &parse_user_header(&req)?, songs.into_inner()).await?))
}

#[delete("/api/songs")]
pub async fn delete(
    req: HttpRequest,
    songs: Json<Vec<Song>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::NoContent().json(
        Model::delete(
            db.clone(),
            UserModel::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            songs.into_inner(),
        )
        .await?,
    ))
}
