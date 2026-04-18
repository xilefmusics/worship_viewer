use shared::api::{ApiClient, ListQuery};
use shared::net::DefaultHttpClient;
use shared::user::CreateUser;

use crate::commands::UsersCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_users(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    cmd: &UsersCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        UsersCommand::List { page, page_size } => {
            let mut query = ListQuery::new();
            if let Some(p) = *page {
                query = query.with_page(p);
            }
            if let Some(ps) = *page_size {
                query = query.with_page_size(ps);
            }
            let users = client.list_users(query).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&users),
                _ => output::print_json(&users, &output),
            }
        }
        UsersCommand::Get { id } => {
            validate_resource_id(&id)?;
            let user = client.get_user(&id).await?;
            output::print_json(&user, &output)
        }
        UsersCommand::Create { json } => {
            let payload: CreateUser = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": "api/v1/users",
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let user = client.create_user(payload).await?;
            output::print_json(&user, &output)
        }
        UsersCommand::Delete { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/users/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            client.delete_user(&id).await?;
            output::print_json(&serde_json::json!({"deleted": true}), &output)
        }
    }
}
