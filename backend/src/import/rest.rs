use super::import;
use crate::error::AppError;

use actix_web::{get, web::Path, HttpResponse};

#[get("/api/import/{identifier:.*}")]
pub async fn get(identifier: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(import(&identifier).await?))
}
