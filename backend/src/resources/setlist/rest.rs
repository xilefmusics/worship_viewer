use actix_web::{
    HttpResponse, Scope, delete, get, post, put,
    web::{self, Data, Json, Path, ReqData},
};

use super::Model;
use crate::database::Database;
#[allow(unused_imports)]
use crate::docs::ErrorResponse;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::setlist::CreateSetlist;
#[allow(unused_imports)]
use crate::resources::setlist::Setlist;
#[allow(unused_imports)]
use crate::resources::song::Song;
use shared::player::Player;

pub fn scope() -> Scope {
    web::scope("/setlists")
        .service(get_setlists)
        .service(get_setlist)
        .service(get_setlist_songs)
        .service(get_setlist_player)
        .service(create_setlist)
        .service(update_setlist)
        .service(delete_setlist)
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists",
    responses(
        (status = 200, description = "Return all setlists", body = [Setlist]),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlists", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("")]
async fn get_setlists(db: Data<Database>, user: ReqData<User>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_setlists(user.read()).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return a single setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}")]
async fn get_setlist(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_setlist(user.read(), &id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/player",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return player metadata for a setlist", body = Player),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist player data", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/player")]
async fn get_setlist_player(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.get_setlist_songs(user.read(), &id)
            .await?
            .into_iter()
            .map(|song| Player::from(song))
            .try_fold(Player::default(), |acc, result| {
                Ok::<Player, AppError>(acc + result)
            })?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/setlists/{id}/songs",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Return the songs for a setlist", body = [Song]),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to fetch setlist songs", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[get("/{id}/songs")]
async fn get_setlist_songs(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_setlist_songs(user.read(), &id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/setlists",
    request_body = CreateSetlist,
    responses(
        (status = 201, description = "Create a new setlist", body = Setlist),
        (status = 400, description = "Invalid setlist payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to create setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[post("")]
async fn create_setlist(
    db: Data<Database>,
    user: ReqData<User>,
    payload: Json<CreateSetlist>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_setlist(&user.id, payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    request_body = CreateSetlist,
    responses(
        (status = 200, description = "Update an existing setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 500, description = "Failed to update setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[put("/{id}")]
async fn update_setlist(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
    payload: Json<CreateSetlist>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        db.update_setlist(user.write(), &id, payload.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    responses(
        (status = 200, description = "Delete a setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
        (status = 500, description = "Failed to delete setlist", body = ErrorResponse)
    ),
    tag = "Setlists",
    security(
        ("SessionCookie" = []),
        ("SessionToken" = [])
    )
)]
#[delete("/{id}")]
async fn delete_setlist(
    db: Data<Database>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_setlist(user.write(), &id).await?))
}
