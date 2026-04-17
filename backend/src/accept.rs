use actix_web::HttpRequest;
use actix_web::http::header;

/// Returns true when the client accepts JSON player responses (`application/json`,
/// `application/vnd.worship.player+json`, or `*/*`).
pub fn accepts_worship_player_json(req: &HttpRequest) -> bool {
    let Some(accept) = req.headers().get(header::ACCEPT) else {
        return true;
    };
    let Ok(s) = accept.to_str() else {
        return true;
    };
    let s = s.to_ascii_lowercase();
    if s.trim().is_empty() {
        return true;
    }
    s.contains("*/*")
        || s.contains("application/json")
        || s.contains("application/vnd.worship.player+json")
}
