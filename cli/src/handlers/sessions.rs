use shared::api::ApiClient;
use shared::net::DefaultHttpClient;

use crate::commands::SessionsCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_sessions(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    cmd: &SessionsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SessionsCommand::ListMine => {
            let sessions = client.list_my_sessions().await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&sessions),
                _ => output::print_json(&sessions, &output),
            }
        }
        SessionsCommand::GetMine { id } => {
            validate_resource_id(&id)?;
            let session = client.get_my_session(&id).await?;
            output::print_json(&session, &output)
        }
        SessionsCommand::DeleteMine { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/users/me/sessions/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            client.delete_my_session(&id).await?;
            output::print_json(&serde_json::json!({"deleted": true}), &output)
        }
        SessionsCommand::CreateForUser { user_id } => {
            validate_resource_id(&user_id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": format!("api/v1/users/{user_id}/sessions"),
                    "body": {},
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let session = client.create_session_for_user(&user_id).await?;
            output::print_json(&session, &output)
        }
        SessionsCommand::ListForUser { user_id } => {
            validate_resource_id(&user_id)?;
            let sessions = client.list_sessions_for_user(&user_id).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&sessions),
                _ => output::print_json(&sessions, &output),
            }
        }
        SessionsCommand::GetForUser { user_id, id } => {
            validate_resource_id(&user_id)?;
            validate_resource_id(&id)?;
            let session = client.get_session_for_user(&user_id, &id).await?;
            output::print_json(&session, &output)
        }
        SessionsCommand::DeleteForUser { user_id, id } => {
            validate_resource_id(&user_id)?;
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/users/{user_id}/sessions/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            client.delete_session_for_user(&user_id, &id).await?;
            output::print_json(&serde_json::json!({"deleted": true}), &output)
        }
    }
}
