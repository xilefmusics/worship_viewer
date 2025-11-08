use super::user;
use crate::auth::middleware::RequireUser;
use actix_web::{dev::HttpServiceFactory, web};

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .wrap(RequireUser::default())
        .service(user::rest::scope())
}
