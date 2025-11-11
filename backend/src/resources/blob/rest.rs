use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path},
};

use super::Model;
use crate::auth::middleware::RequireAdmin;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::blob::Blob;

pub fn scope() -> Scope {
    web::scope("/blobs")
        .service(get_blobs)
        .service(get_blob)
        .service(
            web::scope("")
                .wrap(RequireAdmin::default())
                .service(create_blob)
                .service(update_blob)
                .service(delete_blob),
        )
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
async fn get_blobs(db: Data<Database>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_blobs().await?))
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
async fn get_blob(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_blob(&id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/blobs",
    request_body = Blob,
    responses(
        (status = 201, description = "Create a new blob", body = Blob),
        (status = 400, description = "Invalid blob payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 500, description = "Failed to create blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_blob(db: Data<Database>, payload: Json<Blob>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_blob(payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    request_body = Blob,
    responses(
        (status = 200, description = "Update an existing blob", body = Blob),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
    id: Path<String>,
    payload: Json<Blob>,
) -> Result<HttpResponse, AppError> {
    let mut blob = payload.into_inner();
    blob.id = Some(id.clone());
    Ok(HttpResponse::Ok().json(db.update_blob(&id, blob).await?))
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
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
async fn delete_blob(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_blob(&id).await?))
}
