use actix_web::{HttpMessage, http::header};

pub fn authorization_bearer(msg: &impl HttpMessage) -> Option<String> {
    let value = msg
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .trim();

    if value.is_empty() {
        return None;
    }

    let (scheme, token) = value.split_once(' ')?;

    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }

    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_owned())
    }
}
