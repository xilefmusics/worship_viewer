use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{Group, GroupDatabase, User, UserDatabase};
use crate::AppError;

use actix_web::{get, web::Data, HttpRequest, HttpResponse};

#[get("/api/groups")]
pub async fn groups(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select_all_with_pagination::<GroupDatabase>("group", 0, 100)
            .await?
            .into_iter()
            .map(|group| group.into())
            .collect::<Vec<Group>>(),
    ))
}

#[get("/api/users")]
pub async fn users(req: HttpRequest, db: Data<Database>) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.select_all_with_pagination::<UserDatabase>("user", 0, 100)
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<User>>(),
    ))
}
