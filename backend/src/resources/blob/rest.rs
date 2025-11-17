use std::io::ErrorKind;
use std::path::Path;

use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path as PathParam, ReqData},
};

use super::Model;
use crate::database::Database;
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::blob::Blob;
use crate::resources::blob::CreateBlob;
use crate::settings::Settings;

pub fn scope() -> Scope {
    web::scope("/blobs")
        .service(get_blobs)
        .service(get_blob)
        .service(create_blob)
        .service(update_blob)
        .service(delete_blob)
        .service(download_blob_image)
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs",
    responses(
        (status = 200, description = "Return all blobs", body = [Blob]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch blobs", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_blobs(db: Data<Database>, user: ReqData<User>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_blobs(user.read()).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    responses(
        (status = 200, description = "Return a single blob", body = Blob),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_blob(
    db: Data<Database>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_blob(user.read(), &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/blobs",
    request_body = CreateBlob,
    responses(
        (status = 201, description = "Create a new blob", body = Blob),
        (status = 400, description = "Invalid blob payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_blob(
    db: Data<Database>,
    user: ReqData<User>,
    payload: Json<CreateBlob>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_blob(&user.id, payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    request_body = CreateBlob,
    responses(
        (status = 200, description = "Update an existing blob", body = Blob),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found", body = ErrorResponse),
        (status = 500, description = "Failed to update blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_blob(
    db: Data<Database>,
    user: ReqData<User>,
    id: PathParam<String>,
    payload: Json<CreateBlob>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.update_blob(user.write(), &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    responses(
        (status = 200, description = "Delete a blob", body = Blob),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_blob(
    db: Data<Database>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_blob(user.write(), &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs/{id}/image",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    responses(
        (status = 200, description = "Download the blob image", body = Vec<u8>),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found", body = ErrorResponse),
        (status = 500, description = "Failed to download blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/image")]
async fn download_blob_image(
    db: Data<Database>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let id = id.into_inner();
    let blob = db.get_blob(user.read(), &id).await?;

    let settings = Settings::global();
    let root = Path::new(&settings.blob_dir);
    let file_path = root.join(format!("{}{}", id, blob.file_type.file_ending()));

    let bytes = tokio::fs::read(&file_path).await.map_err(|err| match err.kind() {
        ErrorKind::NotFound => AppError::NotFound("blob image not found".into()),
        _ => AppError::Internal(format!("failed to read blob image: {}", err)),
    })?;

    Ok(HttpResponse::Ok()
        .content_type(blob.file_type.mime())
        .body(bytes))
}
