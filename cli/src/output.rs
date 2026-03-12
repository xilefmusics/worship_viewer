use std::io::{self, IsTerminal};

use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Auto,
    Json,
    Pretty,
    Ndjson,
}

pub fn is_stdout_tty() -> bool {
    io::stdout().is_terminal()
}

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
