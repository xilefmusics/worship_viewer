use super::{Model, User};
use crate::error::AppError;
use crate::rest::parse_user_header;

use fancy_surreal::Client;

use actix_web::{
    delete, get, post, put, web::Data, web::Json, web::Path, HttpRequest, HttpResponse,
};

#[get("/api/users")]
pub async fn get(req: HttpRequest, db: Data<Client<'_>>) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Ok().json(
        Model::get(
            db.clone(),
            Model::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
        )
        .await?,
    ))
}

#[get("/api/users/{id}")]
pub async fn get_id(
    req: HttpRequest,
    db: Data<Client<'_>>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Ok(HttpResponse::Ok().json(
        Model::get_one(
            db.clone(),
            Model::get_or_create(db, &parse_user_header(&req)?)
                .await?
                .read,
            &id.into_inner(),
        )
        .await?,
    ))
}

#[put("/api/users")]
pub async fn put(
    req: HttpRequest,
    users: Json<Vec<User>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Model::admin_write_or_unauthorized(db.clone(), &parse_user_header(&req)?).await?;
    Ok(HttpResponse::Created()
        .json(Model::put(db.clone(), vec!["admin".to_string()], users.into_inner()).await?))
}

#[post("/api/users")]
pub async fn post(
    req: HttpRequest,
    songs: Json<Vec<User>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Model::admin_write_or_unauthorized(db.clone(), &parse_user_header(&req)?).await?;
    Ok(HttpResponse::Created().json(Model::create(db, songs.into_inner()).await?))
}

#[delete("/api/users")]
pub async fn delete(
    req: HttpRequest,
    users: Json<Vec<User>>,
    db: Data<Client<'_>>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    Model::admin_write_or_unauthorized(db.clone(), &parse_user_header(&req)?).await?;
    Ok(HttpResponse::NoContent()
        .json(Model::delete(db.clone(), vec!["admin".to_string()], users.into_inner()).await?))
}
