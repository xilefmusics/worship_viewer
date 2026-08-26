use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, get, head, put,
    web::{self, Data, Path, Query, ReqData},
};

use std::sync::Arc;

use shared::{MediaAssetKind, MediaUploadResponse};

use crate::auth::AuthorizationContext;
use crate::docs::Problem;
use crate::error::AppError;
use crate::http_range::file_data_response;
use crate::settings::MediaAssetUploadLimits;

use super::service::MediaAssetServiceHandle;

#[derive(serde::Deserialize)]
struct UploadQuery {
    kind: String,
    #[serde(default)]
    replace_page: Option<String>,
}

pub fn upload_scope(limits: MediaAssetUploadLimits) -> Scope {
    web::scope("")
        .app_data(web::PayloadConfig::new(limits.payload_ceiling_bytes))
        .service(upload_media_asset)
}

#[utoipa::path(
    put,
    path = "/api/v1/media/{media_id}/uploads",
    params(
        ("media_id" = String, Path, description = "Media identifier"),
        ("kind" = String, Query, description = "Source kind: video, audio, image, pdf, or svg"),
        ("replace_page" = Option<String>, Query, description = "Existing staged/ready deck page id to replace")
    ),
    request_body(
        content = Vec<u8>,
        content_type = "application/octet-stream",
        description = "Streaming upload body"
    ),
    responses(
        (status = 200, description = "Upload accepted to staging", body = MediaUploadResponse),
        (status = 400, description = "Invalid kind or request", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Media not found or write access denied", body = Problem, content_type = "application/problem+json"),
        (status = 413, description = "Payload too large", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Upload failed", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Media",
    security(("SessionCookie" = []), ("SessionToken" = []))
)]
#[put("/{media_id}/uploads")]
async fn upload_media_asset(
    svc: Data<MediaAssetServiceHandle>,
    processing: Data<Arc<crate::resources::media::processing::MediaProcessingHandle>>,
    ctx: ReqData<AuthorizationContext>,
    media_id: Path<String>,
    query: Query<UploadQuery>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, AppError> {
    let kind = MediaAssetKind::parse(&query.kind)
        .ok_or_else(|| AppError::invalid_request("invalid media asset kind"))?;
    let replace_page = query.replace_page.clone();
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_owned();
    let content_length = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());
    let body = svc
        .upload_staging_for_user(&ctx, &media_id, kind, content_type, content_length, payload)
        .await?;
    if matches!(
        kind,
        MediaAssetKind::Video
            | MediaAssetKind::Audio
            | MediaAssetKind::Image
            | MediaAssetKind::Pdf
            | MediaAssetKind::Svg
    ) {
        processing
            .begin_after_upload(&media_id, &body.operation_id, kind, replace_page)
            .await?;
    }
    Ok(HttpResponse::Ok().json(body))
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{media_id}/assets/{asset_id}/data",
    params(
        ("media_id" = String, Path),
        ("asset_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Final asset bytes"),
        (status = 206, description = "Partial content"),
        (status = 304, description = "Not modified"),
        (status = 404, description = "Not found or concealed", body = Problem, content_type = "application/problem+json"),
        (status = 416, description = "Range not satisfiable", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Media",
    security(("SessionCookie" = []), ("SessionToken" = []))
)]
#[get("/{media_id}/assets/{asset_id}/data")]
pub async fn get_media_asset_data(
    req: HttpRequest,
    svc: Data<MediaAssetServiceHandle>,
    ctx: ReqData<AuthorizationContext>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (media_id, asset_id) = path.into_inner();
    let asset = svc
        .get_final_asset_for_user(&ctx, &media_id, &asset_id)
        .await?;
    let etag = asset
        .etag
        .as_deref()
        .ok_or_else(|| AppError::Internal("asset missing etag".into()))?;
    let file_path = svc.final_file_path(&asset_id);
    file_data_response(&req, &file_path, &asset.content_type, etag, false).await
}

#[utoipa::path(
    head,
    path = "/api/v1/media/{media_id}/assets/{asset_id}/data",
    params(
        ("media_id" = String, Path),
        ("asset_id" = String, Path)
    ),
    responses(
        (status = 200, description = "Final asset metadata"),
        (status = 206, description = "Partial content metadata"),
        (status = 304, description = "Not modified"),
        (status = 404, description = "Not found or concealed", body = Problem, content_type = "application/problem+json"),
        (status = 416, description = "Range not satisfiable", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Media",
    security(("SessionCookie" = []), ("SessionToken" = []))
)]
#[head("/{media_id}/assets/{asset_id}/data")]
pub async fn head_media_asset_data(
    req: HttpRequest,
    svc: Data<MediaAssetServiceHandle>,
    ctx: ReqData<AuthorizationContext>,
    path: Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (media_id, asset_id) = path.into_inner();
    let asset = svc
        .get_final_asset_for_user(&ctx, &media_id, &asset_id)
        .await?;
    let etag = asset
        .etag
        .as_deref()
        .ok_or_else(|| AppError::Internal("asset missing etag".into()))?;
    let file_path = svc.final_file_path(&asset_id);
    file_data_response(&req, &file_path, &asset.content_type, etag, true).await
}
