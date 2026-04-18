use actix_web::http::header;
use actix_web::{
    HttpRequest, HttpResponse, Scope, delete, get, patch, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

use crate::accept::accepts_worship_player_json;
#[allow(unused_imports)]
use crate::docs::Problem;
use crate::error::AppError;
use crate::http_cache::{check_if_match, if_none_match_matches, weak_etag_json};
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::collection::Collection;
use crate::resources::collection::PatchCollection;
use crate::resources::collection::service::CollectionServiceHandle;
use crate::resources::collection::{CreateCollection, UpdateCollection};
#[allow(unused_imports)]
use crate::resources::song::Song;
use crate::resources::team::UserPermissions;
use shared::api::{ListQuery, PAGE_SIZE_DEFAULT, PageQuery};
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/collections")
        .service(get_collections)
        .service(get_collection)
        .service(get_collection_songs)
        .service(get_collection_player)
        .service(create_collection)
        .service(update_collection)
        .service(patch_collection)
        .service(delete_collection)
}

#[utoipa::path(
    get,
    path = "/api/v1/collections",
    params(
        ("page" = Option<u32>, Query, description = "Zero-based page (default 0). `X-Total-Count` = filtered total before pagination; last page when `items.len() < page_size` or empty (`list-pagination.md`).", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50.", minimum = 1, maximum = 500, example = 50, nullable = true),
        ("q" = Option<String>, Query, description = "Full-text search query (title); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all collections. `X-Total-Count` header contains the total number of matching collections.", body = [Collection]),
        (status = 400, description = "Invalid pagination parameters", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch collections", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_collections(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let q_ref = query.q.clone();
    let q_link = query.clone();
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let collections = svc.list_collections_for_user(&perms, query).await?;
    let total = svc
        .count_collections_for_user(&perms, q_ref.as_deref())
        .await?;
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
        .json(collections))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 200, description = "Return a single collection (weak `ETag`; `If-None-Match` supported)", body = Collection),
        (status = 304, description = "Not modified"),
        (status = 400, description = "Invalid collection identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch collection", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_collection(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let collection = svc.get_collection_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&collection).map_err(|e| AppError::internal_from_err("collection.rest", e))?;
    if if_none_match_matches(&req, &etag) {
        return Ok(HttpResponse::NotModified()
            .insert_header((header::ETAG, etag))
            .finish());
    }
    Ok(HttpResponse::Ok()
        .insert_header((header::ETAG, etag))
        .json(collection))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/player",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a collection", body = Player),
        (status = 400, description = "Invalid collection identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 406, description = "No supported representation in Accept header", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch collection player data", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_collection_player(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    if !accepts_worship_player_json(&req) {
        return Err(AppError::not_acceptable(
            "supported Accept values include application/json, application/vnd.worship.player+json, and */*",
        ));
    }
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.collection_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/songs",
    params(
        ("id" = String, Path, description = "Collection identifier"),
        ("page" = Option<u32>, Query, description = "Page index, zero-based. Omit with `page_size` for full list.", minimum = 0, nullable = true),
        ("page_size" = Option<u32>, Query, description = "Items per page. Must be 1–500. Defaults to 50. Omit with `page` for full list.", minimum = 1, maximum = 500, example = 50, nullable = true),
    ),
    responses(
        (status = 200, description = "Return the songs for a collection. `X-Total-Count` is the total before paging.", body = [Song]),
        (status = 400, description = "Invalid collection identifier or pagination", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to fetch collection songs", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/songs")]
async fn get_collection_songs(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    query: Query<PageQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query
        .into_inner()
        .validate()
        .map_err(crate::error::map_list_query_error)?;
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let q_link = query.clone();
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(PAGE_SIZE_DEFAULT);
    let (songs, total) = svc
        .collection_songs_for_user(&perms, &id, query.as_list_query())
        .await?;
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
        .json(songs))
}

#[utoipa::path(
    post,
    path = "/api/v1/collections",
    request_body = CreateCollection,
    responses(
        (status = 201, description = "Create a new collection", body = Collection),
        (status = 400, description = "Invalid collection payload", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to create collection", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_collection(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    payload: Json<CreateCollection>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    Ok(HttpResponse::Created().json(
        svc.create_collection_for_user(&perms, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/collections/{id}",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    request_body = UpdateCollection,
    responses(
        (status = 200, description = "Replace collection fields (`PUT` is full replacement, not upsert; missing id returns **404**).", body = Collection),
        (status = 400, description = "Invalid collection identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to update collection", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_collection(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<UpdateCollection>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let collection = svc.get_collection_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&collection).map_err(|e| AppError::internal_from_err("collection.rest", e))?;
    check_if_match(&req, &etag)?;
    let payload = CreateCollection::from(payload.into_inner());
    Ok(HttpResponse::Ok().json(svc.update_collection_for_user(&perms, &id, payload).await?))
}

#[utoipa::path(
    patch,
    path = "/api/v1/collections/{id}",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    request_body = PatchCollection,
    responses(
        (status = 200, description = "Partially update an existing collection", body = Collection),
        (status = 400, description = "Invalid collection identifier or payload", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to patch collection", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[patch("/{id}")]
async fn patch_collection(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<PatchCollection>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let collection = svc.get_collection_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&collection).map_err(|e| AppError::internal_from_err("collection.rest", e))?;
    check_if_match(&req, &etag)?;
    Ok(HttpResponse::Ok().json(
        svc.patch_collection_for_user(&perms, &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/collections/{id}",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 204, description = "Collection deleted"),
        (status = 400, description = "Invalid collection identifier", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "Authentication required", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "API rate limit exceeded; see `Retry-After` and `X-RateLimit-*` response headers", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Collection not found", body = Problem, content_type = "application/problem+json"),
        (status = 412, description = "`If-Match` does not match current weak ETag", body = Problem, content_type = "application/problem+json"),
        (status = 500, description = "Failed to delete collection", body = Problem, content_type = "application/problem+json")
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_collection(
    req: HttpRequest,
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::from_ref(&user, &svc.teams);
    let id = id.into_inner();
    let collection = svc.get_collection_for_user(&perms, &id).await?;
    let etag = weak_etag_json(&collection).map_err(|e| AppError::internal_from_err("collection.rest", e))?;
    check_if_match(&req, &etag)?;
    svc.delete_collection_for_user(&perms, &id).await?;
    Ok(HttpResponse::NoContent().finish())
}
