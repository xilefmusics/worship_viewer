use actix_files::NamedFile;
use actix_web::{get, web::Path};
use std::path::PathBuf;

use crate::error::AppError;
use crate::settings::Settings;

#[get("/")]
pub async fn get_index() -> Result<NamedFile, AppError> {
    let root_path = PathBuf::from(&Settings::global().static_dir);
    let file_path = PathBuf::from("index.html");
    NamedFile::open(root_path.join(file_path)).map_err(|err| AppError::NotFound(err.to_string()))
}

#[get("/{path}")]
pub async fn get_static_files(path: Path<String>) -> Result<NamedFile, AppError> {
    let root_path = PathBuf::from(&Settings::global().static_dir);
    let file_path = PathBuf::from(path.into_inner());
    let path = root_path.join(file_path);
    if path.extension().is_some() {
        NamedFile::open(path).map_err(|err| AppError::NotFound(err.to_string()))
    } else {
        NamedFile::open(root_path.join("index.html"))
            .map_err(|err| AppError::NotFound(err.to_string()))
    }
}
