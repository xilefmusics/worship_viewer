use std::io::{self, IsTerminal};

use clap::ValueEnum;
use serde::Serialize;

/// Output format for JSON printed by the CLI.
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Automatically choose a format based on whether stdout is a TTY.
    Auto,
    /// Compact JSON, one document per command.
    Json,
    /// Pretty-printed JSON.
    Pretty,
    /// Newline-delimited JSON (only meaningful for list commands).
    Ndjson,
}

pub fn is_stdout_tty() -> bool {
    io::stdout().is_terminal()
}

/// Resolve Auto to Json or Pretty based on TTY; other variants unchanged.
pub fn effective_output_format(format: &OutputFormat) -> OutputFormat {
    match format {
        OutputFormat::Auto => {
            if is_stdout_tty() {
                OutputFormat::Pretty
            } else {
                OutputFormat::Json
            }
        }
        other => other.clone(),
    }
}

pub fn print_json<T: Serialize>(
    value: &T,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = effective_output_format(format);
    match fmt {
        OutputFormat::Json | OutputFormat::Auto => {
            let json = serde_json::to_string(value)?;
            println!("{json}");
        }
        OutputFormat::Pretty => {
            let json = serde_json::to_string_pretty(value)?;
            println!("{json}");
        }
        OutputFormat::Ndjson => {
            let json = serde_json::to_string(value)?;
            println!("{json}");
        }
    }
    Ok(())
}

pub fn print_ndjson_list<T: Serialize>(
    values: &[T],
) -> Result<(), Box<dyn std::error::Error>> {
    for v in values {
        let json = serde_json::to_string(v)?;
        println!("{json}");
    }
    Ok(())
}
