use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Bytes, Data, Json, Path as PathParam, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::http_cache::{if_none_match_matches, weak_etag_json};
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::blob::Blob;
use crate::resources::blob::CreateBlob;
use crate::resources::blob::PatchBlob;
use crate::resources::blob::service::BlobServiceHandle;
use crate::resources::team::UserPermissions;
use shared::api::ListQuery;

pub fn scope(blob_upload_max_bytes: usize) -> Scope {
    web::scope("/blobs")
        .service(get_blobs)
        .service(get_blob)
        .service(create_blob)
        .service(update_blob)
        .service(patch_blob)
        .service(delete_blob)
        .service(download_blob_image)
        .service(
            web::scope("")
                .app_data(web::PayloadConfig::new(blob_upload_max_bytes))
                .service(upload_blob_data),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs",
    params(
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Defaults to 0."),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50.")
    ),
    responses(
        (status = 200, description = "Return all blobs. `X-Total-Count` header contains the total number of blobs.", body = [Blob]),
        (status = 400, description = "Invalid pagination parameters", body = ErrorResponse),
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
    let query = query
        .into_inner()
        .validate()
        .map_err(AppError::invalid_request)?;
    let perms = UserPermissions::new(&user, &svc.teams);
    let blobs = svc.list_blobs_for_user(&perms, query).await?;
    let total = svc.count_blobs_for_user(&perms).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .json(blobs))
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs/{id}",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    responses(
        (status = 200, description = "Return a single blob (weak `ETag`; `If-None-Match` supported)", body = Blob),
        (status = 304, description = "Not modified"),
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
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::Internal(e.to_string()))?;
    if if_none_match_matches(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .finish());
    }
    Ok(HttpResponse::Ok()
        .insert_header((header::ETAG, etag))
        .json(blob))
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
        (status = 204, description = "Blob deleted"),
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
    svc.delete_blob_for_user(&perms, &id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/blobs/{id}/data",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    responses(
        (
            status = 200,
            description = "Binary image data. `Content-Type` reflects the stored file type \
                           (`image/png`, `image/jpeg`, or `image/svg`).",
            content_type = "image/*",
            body = Vec<u8>
        ),
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
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    let id = id.into_inner();
    let (blob, file) = svc.open_blob_data_file_for_user(&perms, &id).await?;
    let filename = blob
        .file_name()
        .unwrap_or_else(|| format!("blob-{}", blob.id));
    let mut res = file.into_response(&req);
    res.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            filename.replace('\\', "\\\\").replace('"', "\\\"")
        ))
        .map_err(|e| AppError::Internal(e.to_string()))?,
    );
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("private, max-age=3600, immutable"),
    );
    Ok(res)
}

#[utoipa::path(
    put,
    path = "/api/v1/blobs/{id}/data",
    params(
        ("id" = String, Path, description = "Blob identifier")
    ),
    request_body(
        content = Vec<u8>,
        content_type = "application/octet-stream",
        description = "Raw binary content to store for this blob"
    ),
    responses(
        (status = 204, description = "Blob content uploaded successfully"),
        (status = 400, description = "Invalid blob identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Blob not found or write access denied", body = ErrorResponse),
        (status = 413, description = "Payload too large", body = ErrorResponse),
        (status = 500, description = "Failed to store blob content", body = ErrorResponse)
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}/data")]
async fn upload_blob_data(
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    body: Bytes,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    svc.upload_blob_data_for_user(&perms, &id, &body).await?;
    Ok(HttpResponse::NoContent().finish())
}
