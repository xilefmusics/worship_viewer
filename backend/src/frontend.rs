pub mod rest {
    use actix_files::{Files, NamedFile};
    use actix_web::dev::{ServiceRequest, ServiceResponse, fn_service};
    use actix_web::{Error as ActixError, Scope, get, web};
    use std::path::PathBuf;

    use crate::error::AppError;
    use crate::settings::Settings;

    fn index_file() -> Result<NamedFile, AppError> {
        let root_path = PathBuf::from(&Settings::global().static_dir);
        let file_path = PathBuf::from("index.html");
        NamedFile::open(root_path.join(file_path))
            .map_err(|err| AppError::NotFound(err.to_string()))
    }

    #[get("/")]
    pub async fn get_index() -> Result<NamedFile, AppError> {
        index_file()
    }

    async fn spa_fallback(req: ServiceRequest) -> Result<ServiceResponse, ActixError> {
        let index = index_file()?;
        let (req, _) = req.into_parts();
        let response = index.into_response(&req);
        Ok(ServiceResponse::new(req, response))
    }

    pub fn scope() -> Scope {
        let static_dir = Settings::global().static_dir.clone();

        web::scope("").service(get_index).service(
            Files::new("/", static_dir)
                .index_file("index.html")
                .default_handler(fn_service(spa_fallback)),
        )
    }
}
