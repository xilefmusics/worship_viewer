use shared::api::ApiClient;
use shared::net::DefaultHttpClient;
use shared::team::{CreateTeam, UpdateTeam};

use crate::commands::TeamsCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_teams(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    cmd: &TeamsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        TeamsCommand::List => {
            let teams = client.list_teams().await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&teams),
                _ => output::print_json(&teams, &output),
            }
        }
        TeamsCommand::Get { id } => {
            validate_resource_id(&id)?;
            let team = client.get_team(&id).await?;
            output::print_json(&team, &output)
        }
        TeamsCommand::Create { json } => {
            let payload: CreateTeam = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": "api/v1/teams",
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let team = client.create_team(payload).await?;
            output::print_json(&team, &output)
        }
        TeamsCommand::Update { id, json } => {
            validate_resource_id(&id)?;
            let payload: UpdateTeam = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "PUT",
                    "path": format!("api/v1/teams/{id}"),
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let team = client.update_team(&id, payload).await?;
            output::print_json(&team, &output)
        }
        TeamsCommand::Delete { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/teams/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let team = client.delete_team(&id).await?;
            output::print_json(&team, &output)
        }
    }
}
