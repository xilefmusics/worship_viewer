use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::collection::Collection;
use crate::resources::collection::CreateCollection;
use crate::resources::collection::service::CollectionServiceHandle;
#[allow(unused_imports)]
use crate::resources::song::{Format, QueryParams, Song};
use crate::resources::team::UserPermissions;
use shared::api::ListQuery;
#[allow(unused_imports)]
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/collections")
        .service(get_collections)
        .service(get_collection)
        .service(get_collection_songs)
        .service(get_collection_player)
        .service(get_collection_export)
        .service(create_collection)
        .service(update_collection)
        .service(delete_collection)
}

#[utoipa::path(
    get,
    path = "/api/v1/collections",
    params(
        ("page" = Option<u32>, Query, description = "Optional page index (zero-based)"),
        ("page_size" = Option<u32>, Query, description = "Optional page size (number of items per page)"),
        ("q" = Option<String>, Query, description = "Full-text search query (title); uses text_search analyzer (stemming)")
    ),
    responses(
        (status = 200, description = "Return all collections", body = [Collection]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch collections", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_collections(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    query: Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.list_collections_for_user(&perms, query.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 200, description = "Return a single collection", body = Collection),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch collection", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_collection(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.get_collection_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/player",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a collection", body = Player),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch collection player data", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_collection_player(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.collection_player_for_user(&perms, &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/export",
    params(
        ("id" = String, Path, description = "Collection identifier"),
        ("format" = Format, Query, description = "Optional export format: zip, wp, cp, pdf (defaults to wp)")
    ),
    responses(
        (
            status = 200,
            description = "Download exported collection file",
            body = Vec<u8>,
            content_type = "application/octet-stream"
        ),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to export collection", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/export")]
async fn get_collection_export(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    query: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    svc.export_collection_for_user(&perms, &id, query.into_inner().format)
        .await
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/songs",
    params(
        ("id" = String, Path, description = "Collection identifier")
    ),
    responses(
        (status = 200, description = "Return the songs for a collection", body = [Song]),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch collection songs", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/songs")]
async fn get_collection_songs(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.collection_songs_for_user(&perms, &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/collections",
    request_body = CreateCollection,
    responses(
        (status = 201, description = "Create a new collection", body = Collection),
        (status = 400, description = "Invalid collection payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create collection", body = ErrorResponse)
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
    let perms = UserPermissions::new(&user, &svc.teams);
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
    request_body = CreateCollection,
    responses(
        (status = 200, description = "Update an existing collection", body = Collection),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to update collection", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_collection(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateCollection>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(
        svc.update_collection_for_user(&perms, &id, payload.into_inner())
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
        (status = 200, description = "Delete a collection", body = Collection),
        (status = 400, description = "Invalid collection identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete collection", body = ErrorResponse)
    ),
    tag = "Collections",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_collection(
    svc: Data<CollectionServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.delete_collection_for_user(&perms, &id).await?))
}
