use crate::database::Database;
use crate::rest::helper::{expect_admin, parse_user_header};
use crate::types::{Group, GroupDatabase, User, UserDatabase};
use crate::AppError;

use actix_web::{post, web::Data, web::Json, HttpRequest, HttpResponse};

#[post("/api/groups")]
pub async fn groups(
    req: HttpRequest,
    groups: Json<Vec<Group>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "group",
            groups
                .clone()
                .into_iter()
                .map(|group| GroupDatabase::try_from(group))
                .collect::<Result<Vec<GroupDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|group| group.into())
        .collect::<Vec<Group>>(),
    ))
}

#[post("/api/users")]
pub async fn users(
    req: HttpRequest,
    users: Json<Vec<User>>,
    db: Data<Database>,
) -> Result<HttpResponse, AppError> {
    expect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(
        db.create_vec(
            "user",
            users
                .clone()
                .into_iter()
                .map(|user| UserDatabase::try_from(user))
                .collect::<Result<Vec<UserDatabase>, AppError>>()?,
        )
        .await?
        .into_iter()
        .map(|user| user.into())
        .collect::<Vec<User>>(),
    ))
}
