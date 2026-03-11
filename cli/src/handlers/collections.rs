use shared::api::ApiClient;
use shared::collection::CreateCollection;
use shared::net::DefaultHttpClient;

use crate::commands::CollectionsCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_collections(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    effective_base_url: &str,
    cmd: &CollectionsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        CollectionsCommand::List => {
            let collections = client.list_collections().await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&collections),
                _ => output::print_json(&collections, &output),
            }
        }
        CollectionsCommand::Get { id } => {
            validate_resource_id(&id)?;
            let collection = client.get_collection(&id).await?;
            output::print_json(&collection, &output)
        }
        CollectionsCommand::Songs { id } => {
            validate_resource_id(&id)?;
            let songs = client.get_collection_songs(&id).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&songs),
                _ => output::print_json(&songs, &output),
            }
        }
        CollectionsCommand::Player { id } => {
            validate_resource_id(&id)?;
            let player = client.get_collection_player(&id).await?;
            output::print_json(&player, &output)
        }
        CollectionsCommand::ExportUrl { id, format } => {
            validate_resource_id(&id)?;
            let url_path = client.get_collection_export_url(&id, &format).await;
            let full_url = format!(
                "{}/{}",
                effective_base_url.trim_end_matches('/'),
                url_path.trim_start_matches('/')
            );
            output::print_json(&serde_json::json!({ "url": full_url }), &output)
        }
        CollectionsCommand::Create { json } => {
            let payload: CreateCollection = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": "api/v1/collections",
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let collection = client.create_collection(payload).await?;
            output::print_json(&collection, &output)
        }
        CollectionsCommand::Update { id, json } => {
            validate_resource_id(&id)?;
            let payload: CreateCollection = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "PUT",
                    "path": format!("api/v1/collections/{id}"),
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let collection = client.update_collection(&id, payload).await?;
            output::print_json(&collection, &output)
        }
        CollectionsCommand::Delete { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/collections/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let collection = client.delete_collection(&id).await?;
            output::print_json(&collection, &output)
        }
    }
}
