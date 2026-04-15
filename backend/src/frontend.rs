pub mod rest {
    use actix_files::{Files, NamedFile};
    use actix_web::dev::{ServiceRequest, ServiceResponse, fn_service};
    use actix_web::{Error as ActixError, Scope, web};
    use std::path::PathBuf;

    use crate::error::AppError;

    fn index_file(static_dir: &str) -> Result<NamedFile, AppError> {
        let root_path = PathBuf::from(static_dir);
        NamedFile::open(root_path.join("index.html"))
            .map_err(|err| AppError::NotFound(err.to_string()))
    }

    pub fn scope(static_dir: &str) -> Scope {
        let static_dir = static_dir.to_owned();

        let spa_fallback = {
            let dir = static_dir.clone();
            move |req: ServiceRequest| {
                let dir = dir.clone();
                async move {
                    let index = index_file(&dir).map_err(ActixError::from)?;
                    let (http_req, _) = req.into_parts();
                    let response = index.into_response(&http_req);
                    Ok(ServiceResponse::new(http_req, response))
                }
            }
        };

        web::scope("").service(
            Files::new("/", static_dir)
                .index_file("index.html")
                .default_handler(fn_service(spa_fallback)),
        )
    }
}
