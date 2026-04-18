//! Tracing subscriber setup (`tracing` + `tracing-subscriber` + `tracing-log` bridge).

use hex;
use ring::digest;
use surrealdb::Error as SurrealError;
use tracing_subscriber::EnvFilter;

/// Compile-time guard: audit [`crate::audit!`] events must use the `audit.` prefix.
pub const fn audit_event_name_ok(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 6
        && b[0] == b'a'
        && b[1] == b'u'
        && b[2] == b'd'
        && b[3] == b'i'
        && b[4] == b't'
        && b[5] == b'.'
}

/// SHA-256 hex digest of `email` for audit logs (never log raw email on failure paths).
pub fn audit_email_hash(email: &str) -> String {
    let d = digest::digest(&digest::SHA256, email.as_bytes());
    hex::encode(d.as_ref())
}

/// Structured audit line: sets `audit = true` and enforces `event` literals starting with `audit.`.
///
/// Fields use `ident = expr` only (use [`tracing::field::display`] / [`tracing::field::debug`] in the expr).
#[macro_export]
macro_rules! audit {
    ($event:literal ; $msg:literal) => {
        const _: () = assert!($crate::observability::audit_event_name_ok($event));
        tracing::info!(audit = true, event = $event, $msg);
    };
    ($event:literal, $($key:ident = $value:expr),+ ; $msg:literal) => {
        const _: () = assert!($crate::observability::audit_event_name_ok($event));
        tracing::info!(audit = true, event = $event, $($key = $value),+, $msg);
    };
}

/// Matches production detection in `main` (initial admin / unsafe dev checks).
pub fn is_production() -> bool {
    matches!(
        std::env::var("WORSHIP_PRODUCTION").ok().as_deref(),
        Some("true" | "1" | "yes")
    ) || matches!(
        std::env::var("RUST_ENV").ok().as_deref(),
        Some("production")
    )
}

#[derive(Clone, Copy)]
enum LogFormat {
    Json,
    Compact,
    Pretty,
}

fn resolve_log_format() -> LogFormat {
    match std::env::var("LOG_FORMAT").ok().as_deref() {
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            "pretty" => LogFormat::Pretty,
            _ => {
                if is_production() {
                    LogFormat::Json
                } else {
                    LogFormat::Compact
                }
            }
        },
        None => {
            if is_production() {
                LogFormat::Json
            } else {
                LogFormat::Compact
            }
        }
    }
}

/// Installs the global `tracing` subscriber and bridges `log` crate output into it.
///
/// Call once at process startup. Safe to call from tests if no other subscriber is set;
/// `tracing-log` init is best-effort if the global logger is already configured.
pub fn init() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match resolve_log_format() {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(false)
                .with_target(true)
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .with_target(true)
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .pretty()
                .with_target(true)
                .init();
        }
    }

    let _ = tracing_log::LogTracer::init();
    Ok(())
}

/// Joins [`std::error::Error::source`] links with `" <- "` (top-level message first).
pub fn error_source_chain_string(err: &(dyn std::error::Error + 'static)) -> String {
    let mut sources: Vec<String> = Vec::new();
    sources.push(err.to_string());
    let mut cursor = err.source();
    while let Some(inner) = cursor {
        sources.push(inner.to_string());
        cursor = inner.source();
    }
    sources.join(" <- ")
}

/// Logs `%err`, `?err`, and the full source chain under a stable `target` field.
pub fn log_error_chain(target: &'static str, err: &(dyn std::error::Error + 'static)) {
    tracing::error!(
        target = target,
        error = %err,
        error_source_chain = %error_source_chain_string(err),
        error_debug = ?err,
        "I/O boundary error"
    );
}

/// SurrealDB statement failure during migrations (`migration` field preserved for log compatibility).
pub fn log_surreal_statement_error_migration(
    migration: &str,
    statement_index: usize,
    err: &SurrealError,
) {
    tracing::error!(
        migration = %migration,
        statement_index = statement_index,
        error = %err,
        error_source_chain = %error_source_chain_string(err),
        error_debug = ?err,
        "SurrealDB query statement failed"
    );
}

/// SurrealDB statement failure in application queries (`context` field).
pub fn log_surreal_statement_error_context(
    context: &'static str,
    statement_index: usize,
    err: &SurrealError,
) {
    tracing::error!(
        context = context,
        statement_index = statement_index,
        error = %err,
        error_source_chain = %error_source_chain_string(err),
        error_debug = ?err,
        "SurrealDB query statement failed"
    );
}

/// Log a typed error's chain, then convert with `AppError::{database,mail,oidc}` (or similar).
#[macro_export]
macro_rules! log_and_convert {
    ($mapper:path, $target:literal, $err:expr) => {{
        let err = $err;
        $crate::observability::log_error_chain($target, &err);
        $mapper(err)
    }};
}
