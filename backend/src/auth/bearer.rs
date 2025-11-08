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

    let (scheme, token) = match value.split_once(' ') {
        Some(pair) => pair,
        None => return Some(value.to_owned()),
    };

    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }

    if token.trim().is_empty() {
        None
    } else {
        Some(token.trim().to_owned())
    }
}
