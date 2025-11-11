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
use crate::resources::setlist::Setlist;

pub fn scope() -> Scope {
    web::scope("/setlists")
        .service(get_setlists)
        .service(get_setlist)
        .service(
            web::scope("")
                .wrap(RequireAdmin::default())
                .service(create_setlist)
                .service(update_setlist)
                .service(delete_setlist),
        )
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
async fn get_setlists(db: Data<Database>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_setlists().await?))
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
async fn get_setlist(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.get_setlist(&id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/setlists",
    request_body = Setlist,
    responses(
        (status = 201, description = "Create a new setlist", body = Setlist),
        (status = 400, description = "Invalid setlist payload", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
    payload: Json<Setlist>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Created().json(db.create_setlist(payload.into_inner()).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/setlists/{id}",
    params(
        ("id" = String, Path, description = "Setlist identifier")
    ),
    request_body = Setlist,
    responses(
        (status = 200, description = "Update an existing setlist", body = Setlist),
        (status = 400, description = "Invalid setlist identifier", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Setlist not found", body = ErrorResponse),
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
    id: Path<String>,
    payload: Json<Setlist>,
) -> Result<HttpResponse, AppError> {
    let mut setlist = payload.into_inner();
    setlist.id = Some(id.clone());
    Ok(HttpResponse::Ok().json(db.update_setlist(&id, setlist).await?))
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
        (status = 403, description = "Admin role required", body = ErrorResponse),
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
async fn delete_setlist(db: Data<Database>, id: Path<String>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(db.delete_setlist(&id).await?))
}
