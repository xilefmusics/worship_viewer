use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, Query, ReqData},
};

use super::Model;
use crate::database::Database;
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
#[allow(unused_imports)]
use crate::resources::collection::Collection;
use crate::resources::collection::CreateCollection;
#[allow(unused_imports)]
use crate::resources::song::{QueryParams, Song, export};
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
    db: Data<Database>,
    user: ReqData<User>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_collections(user.read()).await?))
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_collection(user.read(), &id).await?))
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.get_collection_songs(user.read(), &id)
            .await?
            .into_iter()
            .enumerate()
            .map(|(idx, song_link_owned)| {
                let mut player = Player::from(song_link_owned.song);
                player.add_numbers(std::iter::once(
                    song_link_owned.nr.unwrap_or_else(|| idx.to_string()),
                ));
                player
            })
            .try_fold(Player::default(), |acc, player| {
                Ok::<Player, AppError>(acc + player)
            })?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/export",
    params(
        ("id" = String, Path, description = "Collection identifier"),
        ("format" = crate::resources::song::Format, Query, description = "Optional export format: zip, wp, cp, pdf (defaults to wp)")
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    query: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    export(
        db.get_collection_songs(user.read(), &id)
            .await?
            .into_iter()
            .map(|song_link_owned| song_link_owned.song)
            .collect::<Vec<Song>>(),
        query.into_inner().format,
    )
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.get_collection_songs(user.read(), &id)
            .await?
            .into_iter()
            .map(|song_link_owned| song_link_owned.song)
            .collect::<Vec<Song>>(),
    ))
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
    db: Data<Database>,
    user: ReqData<User>,
    payload: Json<CreateCollection>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_collection(&user.id, payload.into_inner()).await?))
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateCollection>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.update_collection(user.write(), &id, payload.into_inner())
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
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_collection(user.write(), &id).await?))
}
