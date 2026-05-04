//! Derive `client_origin` / `client_version` for [`crate::http_audit`] from request headers.

/// HTTP header sent by first-party clients: `<product>/<version>`.
pub const X_WORSHIP_CLIENT: &str = "X-Worship-Client";

/// Returns `(client_origin, client_version)` for persistence on `http_request_audit`.
pub fn classify(
    x_worship_client: Option<&str>,
    user_agent: Option<&str>,
    referer: Option<&str>,
) -> (String, Option<String>) {
    if let Some(raw) = x_worship_client
        && let Some((product, ver)) = raw.split_once('/')
    {
        let ver = ver.trim();
        if !ver.is_empty() {
            let origin = match product.trim() {
                "worshipviewer-cli" => "cli",
                "worshipviewer-frontend" => "frontend",
                _ => "unknown",
            }
            .to_string();
            return (origin, Some(ver.to_string()));
        }
    }

    if referer.is_some_and(|r| r.to_ascii_lowercase().contains("/api/docs")) {
        return ("swagger".to_string(), None);
    }

    if user_agent.is_some_and(|ua| ua.contains("reqwest")) {
        return ("cli".to_string(), None);
    }

    if user_agent.is_some_and(|ua| ua.contains("Mozilla")) {
        return ("frontend".to_string(), None);
    }

    ("unknown".to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_cli_header() {
        let (o, v) = classify(Some("worshipviewer-cli/1.2.3"), Some("reqwest/0.12"), None);
        assert_eq!(o, "cli");
        assert_eq!(v.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn primary_frontend_header() {
        let (o, v) = classify(
            Some("worshipviewer-frontend/0.1.0"),
            Some("Mozilla/5.0"),
            None,
        );
        assert_eq!(o, "frontend");
        assert_eq!(v.as_deref(), Some("0.1.0"));
    }

    #[test]
    fn unknown_product_with_version() {
        let (o, v) = classify(Some("other-tool/9.0"), None, None);
        assert_eq!(o, "unknown");
        assert_eq!(v.as_deref(), Some("9.0"));
    }

    #[test]
    fn referer_swagger_wins_over_mozilla_ua() {
        let (o, v) = classify(
            None,
            Some("Mozilla/5.0"),
            Some("http://127.0.0.1:8080/api/docs/"),
        );
        assert_eq!(o, "swagger");
        assert!(v.is_none());
    }

    #[test]
    fn mozilla_without_referer_is_frontend() {
        let (o, v) = classify(
            None,
            Some("Mozilla/5.0 (Macintosh)"),
            Some("https://app.example/"),
        );
        assert_eq!(o, "frontend");
        assert!(v.is_none());
    }

    #[test]
    fn reqwest_without_header() {
        let (o, v) = classify(None, Some("reqwest/0.12"), None);
        assert_eq!(o, "cli");
        assert!(v.is_none());
    }
}
