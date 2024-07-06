use crate::error::AppError;

use actix_files::NamedFile;
use actix_web::{get, web::Path, HttpRequest};
use std::path::PathBuf;

pub fn parse_user_header(req: &HttpRequest) -> Result<String, AppError> {
    Ok(req
        .headers()
        .get("X-Remote-User")
        .ok_or(AppError::Unauthorized("no X-Remote-User given".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("no X-Remote-User given".into()))?
        .into())
}

#[get("/")]
pub async fn get_index() -> Result<NamedFile, AppError> {
    let root_path = PathBuf::from(std::env::var("STATIC_DIR").unwrap_or("static".into()));
    let file_path = PathBuf::from("index.html");
    NamedFile::open(root_path.join(file_path)).map_err(|err| AppError::NotFound(err.to_string()))
}

#[get("/{path}")]
pub async fn get_static_files(path: Path<String>) -> Result<NamedFile, AppError> {
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
