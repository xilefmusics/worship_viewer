//! Async command handlers. Dispatch by command and delegate to per-domain modules.

use shared::api::ApiClient;
use shared::net::DefaultHttpClient;

use crate::commands::{Cli, Command};

mod auth;
mod blobs;
mod collections;
mod schema;
mod sessions;
mod setlists;
mod songs;
mod users;

/// Run the command specified in `cli`, using `client` and `effective_base_url` for API and URL construction.
pub async fn dispatch(
    client: &ApiClient<DefaultHttpClient>,
    cli: &Cli,
    effective_base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Command::Schema { path_prefix } => {
            schema::handle_schema(client, cli.output.clone(), path_prefix.clone()).await
        }
        Command::Auth { command } => {
            auth::handle_auth(client, cli.output.clone(), cli.dry_run, command).await
        }
        Command::Users { command } => {
            users::handle_users(client, cli.output.clone(), cli.dry_run, command).await
        }
        Command::Sessions { command } => {
            sessions::handle_sessions(client, cli.output.clone(), cli.dry_run, command).await
        }
        Command::Songs { command } => {
            songs::handle_songs(
                client,
                cli.output.clone(),
                cli.dry_run,
                effective_base_url,
                command,
            )
            .await
        }
        Command::Collections { command } => {
            collections::handle_collections(
                client,
                cli.output.clone(),
                cli.dry_run,
                effective_base_url,
                command,
            )
            .await
        }
        Command::Setlists { command } => {
            setlists::handle_setlists(
                client,
                cli.output.clone(),
                cli.dry_run,
                effective_base_url,
                command,
            )
            .await
        }
        Command::Blobs { command } => {
            blobs::handle_blobs(
                client,
                cli.output.clone(),
                cli.dry_run,
                effective_base_url,
                command,
            )
            .await
        }
    }
}
