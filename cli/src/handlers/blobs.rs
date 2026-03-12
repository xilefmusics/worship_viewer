use shared::api::{ApiClient, ListQuery};
use shared::blob::CreateBlob;
use shared::net::DefaultHttpClient;

use crate::commands::BlobsCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_blobs(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    effective_base_url: &str,
    cmd: &BlobsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        BlobsCommand::List { page, page_size } => {
            let mut query = ListQuery::new();
            if let Some(p) = *page {
                query = query.with_page(p);
            }
            if let Some(ps) = *page_size {
                query = query.with_page_size(ps);
            }
            let blobs = client.list_blobs(query).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&blobs),
                _ => output::print_json(&blobs, &output),
            }
        }
        BlobsCommand::Get { id } => {
            validate_resource_id(&id)?;
            let blob = client.get_blob(&id).await?;
            output::print_json(&blob, &output)
        }
        BlobsCommand::Create { json } => {
            let payload: CreateBlob = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": "api/v1/blobs",
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let blob = client.create_blob(payload).await?;
            output::print_json(&blob, &output)
        }
        BlobsCommand::Update { id, json } => {
            validate_resource_id(&id)?;
            let payload: CreateBlob = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "PUT",
                    "path": format!("api/v1/blobs/{id}"),
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let blob = client.update_blob(&id, payload).await?;
            output::print_json(&blob, &output)
        }
        BlobsCommand::Delete { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/blobs/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let blob = client.delete_blob(&id).await?;
            output::print_json(&blob, &output)
        }
        BlobsCommand::DownloadUrl { id } => {
            validate_resource_id(&id)?;
            let url_path = client.download_blob_image_url(&id).await;
            let full_url = format!(
                "{}/{}",
                effective_base_url.trim_end_matches('/'),
                url_path.trim_start_matches('/')
            );
            output::print_json(&serde_json::json!({ "url": full_url }), &output)
        }
    }
}
