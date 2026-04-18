//! Parse `?expand=` (comma-separated tokens) for optional relation embedding.

/// True when `expand` includes the `user` token (e.g. `expand=user` or `expand=team,user`).
pub fn expand_includes_user(expand: &Option<String>) -> bool {
    expand
        .as_deref()
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .any(|t| t == "user")
        })
        .unwrap_or(false)
}
