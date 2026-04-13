use super::{blob, collection, setlist, song, team, user};
use crate::auth::middleware::RequireUser;
use actix_web::{dev::HttpServiceFactory, web};

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .wrap(RequireUser)
        .service(blob::rest::scope())
        .service(collection::rest::scope())
        .service(setlist::rest::scope())
        .service(song::rest::scope())
        .service(team::rest::scope())
        .service(team::invitations_accept_scope())
        .service(user::rest::scope())
}
