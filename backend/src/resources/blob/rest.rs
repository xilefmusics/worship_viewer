use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Bytes, Data, Json, Path as PathParam, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::http_cache::{
    check_if_match, if_none_match_matches, weak_etag_from_bytes, weak_etag_json,
};
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::blob::Blob;
use crate::resources::blob::PatchBlob;
use crate::resources::blob::service::BlobServiceHandle;
use crate::resources::blob::{CreateBlob, UpdateBlob};
use crate::resources::team::UserPermissions;
use shared::api::{ListQuery, PAGE_SIZE_DEFAULT};

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
        ("page" = Option<u32>, Query, description = "Zero-based page (default 0). Track A: `X-Total-Count` = pre-pagination total; last page when `items.len() < page_size` or empty. See `list-pagination.md`.", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50.", minimum = 1, maximum = 500, example = 50, nullable = true),
        ("q" = Option<String>, Query, description = "Optional case-insensitive substring filter on stored OCR text. Whitespace-only is treated as absent.")
    ),
    responses(
        (status = 200, description = "Return all blobs. `X-Total-Count` matches the filtered total.", body = [Blob]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch blobs", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_blobs(
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let q_link = query.clone();
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let blobs = svc.list_blobs_for_user(&perms, query.clone()).await?;
    let total = svc.count_blobs_for_user(&perms, &query).await?;
    Ok(HttpResponse::Ok()
        .insert_header((
            header::HeaderName::from_static("x-total-count"),
            total.to_string(),
        ))
        .insert_header((
            header::LINK,
            crate::request_link::list_link_header(
                &req,
                |p| q_link.query_string_for_page(p),
                page,
                page_size,
                total,
            ),
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
        (status = 400, description = "Invalid blob identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Blob not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch blob", body = Problem, content_type = "application/problem+json")
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
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::internal_from_err("blob.rest", e))?;
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
        (status = 400, description = "Invalid blob payload", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to create blob", body = Problem, content_type = "application/problem+json")
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
    let perms = UserPermissions::from_ref(&user, &svc.teams);
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
    request_body = UpdateBlob,
    responses(
        (status = 200, description = "Replace blob metadata (`PUT` is full replacement; `owner` is not client-settable).", body = Blob),
        (status = 400, description = "Invalid blob identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Blob not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag on blob metadata", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to update blob", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_blob(
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    payload: Json<UpdateBlob>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::internal_from_err("blob.rest", e))?;
    check_if_match(&req, &etag)?;
    let payload = CreateBlob::from(payload.into_inner());
    Ok(HttpResponse::Ok().json(svc.update_blob_for_user(&perms, &id, payload).await?))
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
        (status = 400, description = "Invalid blob identifier or payload", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Blob not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag on blob metadata", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to patch blob", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[patch("/{id}")]
async fn patch_blob(
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    payload: Json<PatchBlob>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::internal_from_err("blob.rest", e))?;
    check_if_match(&req, &etag)?;
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
        (status = 400, description = "Invalid blob identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Blob not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag on blob metadata", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to delete blob", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_blob(
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::internal_from_err("blob.rest", e))?;
    check_if_match(&req, &etag)?;
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
                           (`image/png`, `image/jpeg`, or `image/svg+xml`).",
            content_type = "image/*",
            body = Vec<u8>
        ),
        (status = 400, description = "Invalid blob identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 304, description = "Not modified (`If-None-Match` matches weak ETag of bytes)"),
        (status = 404, description = "Blob not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to download blob", body = Problem, content_type = "application/problem+json")
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
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let (blob, file) = svc.open_blob_data_file_for_user(&perms, &id).await?;
    let filename = blob
        .file_name()
        .unwrap_or_else(|| format!("blob-{}", blob.id));
    let path = file.path().to_path_buf();
    let bytes = std::fs::read(&path).map_err(|e| AppError::internal_from_err("blob.rest.read_data", e))?;
    let etag = weak_etag_from_bytes(&bytes);
    if if_none_match_matches(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .insert_header((
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("private, max-age=3600, immutable"),
            ))
            .finish());
    }
    let ct = header::HeaderValue::from_static(blob.file_type.mime());
    let cd = header::HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        filename.replace('\\', "\\\\").replace('"', "\\\"")
    ))
    .map_err(|e| AppError::internal_from_err("blob.rest.content_disposition_header", e))?;
    Ok(HttpResponse::Ok()
        .insert_header((header::ETAG, etag))
        .insert_header((header::CONTENT_TYPE, ct))
        .insert_header((header::CONTENT_DISPOSITION, cd))
        .insert_header((
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("private, max-age=3600, immutable"),
        ))
        .insert_header((header::CONTENT_LENGTH, bytes.len().to_string()))
        .body(bytes))
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
        (status = 400, description = "Invalid blob identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Blob not found or write access denied", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag on blob metadata", body = Problem, content_type = "application/problem+json"),
        (status = 413, description = "Payload too large", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to store blob content", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Blobs",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}/data")]
async fn upload_blob_data(
    req: HttpRequest,
    svc: Data<BlobServiceHandle>,
    user: ReqData<User>,
    id: PathParam<String>,
    body: Bytes,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let blob = svc.get_blob_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&blob).map_err(|e| AppError::internal_from_err("blob.rest", e))?;
    check_if_match(&req, &etag)?;
    svc.upload_blob_data_for_user(&perms, &id, &body).await?;
    Ok(HttpResponse::NoContent().finish())
}
