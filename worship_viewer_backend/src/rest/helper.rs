use crate::AppError;

use actix_web::HttpRequest;

pub fn parse_user_header(req: HttpRequest) -> Result<String, AppError> {
    Ok(req
        .headers()
        .get("X-Remote-User")
        .ok_or(AppError::Unauthorized("no X-Remote-User given".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("no X-Remote-User given".into()))?
        .into())
}

pub fn expect_admin(user: &str) -> Result<(), AppError> {
    // TODO: Check if the user has admin rights
    if user == "admin" {
        Ok(())
    } else {
        Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ))
    }
}
