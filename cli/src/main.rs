use clap::Parser;

use shared::api::ApiClient;
use shared::net::DefaultHttpClient;

mod commands;
mod config;
mod handlers;
mod output;
mod validate;

use crate::commands::Cli;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/xilef/Git/worship_viewer/.cursor/debug-3dd3ca.log")
        {
            let _ = writeln!(
                f,
                "{}",
                serde_json::json!({
                    "sessionId":"3dd3ca",
                    "runId":"pre-fix",
                    "hypothesisId":"H1_global_args_not_global",
                    "location":"cli/src/main.rs:run",
                    "message":"Parsed CLI args",
                    "data":{
                        "output": format!("{:?}", cli.output),
                        "dry_run": cli.dry_run,
                        "has_base_url": cli.base_url.is_some(),
                        "command": format!("{:?}", cli.command),
                    },
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0)
                })
            );
        }
    }

    let options = config::BuildConfigOptions::from_cli(&cli);
    let (client_config, effective_base_url) = config::build_http_client_config(&options)?;
    let client = ApiClient::<DefaultHttpClient>::with_default(client_config);

    handlers::dispatch(&client, &cli, &effective_base_url).await
}
