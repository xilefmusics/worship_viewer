use actix_files::NamedFile;
use actix_web::{
    HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path as PathParam, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::blob::Blob;
use crate::resources::blob::CreateBlob;
use crate::resources::blob::PatchBlob;
use crate::resources::blob::service::BlobServiceHandle;
use crate::resources::team::UserPermissions;
use shared::api::ListQuery;

pub fn scope() -> Scope {
    web::scope("/blobs")
        .service(get_blobs)
        .service(get_blob)
        .service(create_blob)
        .service(update_blob)
        .service(patch_blob)
        .service(delete_blob)
        .service(download_blob_image)
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs",
    params(
        ("page" = Option<u32>, Query, description = "Optional page index (zero-based)"),
        ("page_size" = Option<u32>, Query, description = "Optional page size (number of items per page)")
    ),
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
async fn get_blobs(
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.list_blobs_for_user(&perms, query.into_inner()).await?))
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
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.get_blob_for_user(&perms, &id).await?))
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
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    payload: Json<CreateBlob>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Created().json(
        svc.create_blob_for_user(&perms, payload.into_inner())
            .await?,
    ))
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
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    payload: Json<CreateBlob>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.update_blob_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    request_body = PatchBlob,
    responses(
        (status = 200, description = "Partially update an existing blob", body = Blob),
        (status = 400, description = "Invalid blob identifier or payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found", body = ErrorResponse),
        (status = 500, description = "Failed to patch blob", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[patch("/{id}")]
async fn patch_blob(
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    payload: Json<PatchBlob>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.patch_blob_for_user(&perms, &id, payload.into_inner())
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
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.delete_blob_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs/{id}/data",
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
#[get("/{id}/data")]
async fn download_blob_image(
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<NamedFile, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    svc.open_blob_data_file_for_user(&perms, &id.into_inner())
        .await
}
