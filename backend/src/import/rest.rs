use crate::error::AppError;
use crate::song::Song;

use actix_web::{get, web::Path, HttpRequest, HttpResponse};

#[get("/api/import/{url:.*}")]
pub async fn get(req: HttpRequest, url: Path<String>) -> Result<HttpResponse, AppError> {
    dbg!(url);
    Ok(HttpResponse::Ok().json(Song::default()))
}
