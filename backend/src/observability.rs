//! Tracing subscriber setup (`tracing` + `tracing-subscriber` + `tracing-log` bridge).

use tracing_subscriber::EnvFilter;

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
