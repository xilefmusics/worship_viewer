use super::{blob, collection, setlist, song, user};
use crate::auth::middleware::RequireUser;
use actix_web::{dev::HttpServiceFactory, web};

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .wrap(RequireUser::default())
        .service(blob::rest::scope())
        .service(collection::rest::scope())
        .service(setlist::rest::scope())
        .service(song::rest::scope())
        .service(user::rest::scope())
}
