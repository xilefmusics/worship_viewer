use shared::api::{ApiClient, ListQuery};
use shared::net::DefaultHttpClient;
use shared::setlist::CreateSetlist;

use crate::commands::SetlistsCommand;
use crate::output::{self, OutputFormat};
use crate::validate::validate_resource_id;

pub async fn handle_setlists(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    dry_run: bool,
    effective_base_url: &str,
    cmd: &SetlistsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SetlistsCommand::List { page, page_size } => {
            let mut query = ListQuery::new();
            if let Some(p) = *page {
                query = query.with_page(p);
            }
            if let Some(ps) = *page_size {
                query = query.with_page_size(ps);
            }
            let setlists = client.list_setlists(query).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&setlists),
                _ => output::print_json(&setlists, &output),
            }
        }
        SetlistsCommand::Get { id } => {
            validate_resource_id(&id)?;
            let setlist = client.get_setlist(&id).await?;
            output::print_json(&setlist, &output)
        }
        SetlistsCommand::Songs { id } => {
            validate_resource_id(&id)?;
            let songs = client.get_setlist_songs(&id).await?;
            match output::effective_output_format(&output) {
                OutputFormat::Ndjson => output::print_ndjson_list(&songs),
                _ => output::print_json(&songs, &output),
            }
        }
        SetlistsCommand::Player { id } => {
            validate_resource_id(&id)?;
            let player = client.get_setlist_player(&id).await?;
            output::print_json(&player, &output)
        }
        SetlistsCommand::ExportUrl { id, format } => {
            validate_resource_id(&id)?;
            let url_path = client.get_setlist_export_url(&id, &format).await;
            let full_url = format!(
                "{}/{}",
                effective_base_url.trim_end_matches('/'),
                url_path.trim_start_matches('/')
            );
            output::print_json(&serde_json::json!({ "url": full_url }), &output)
        }
        SetlistsCommand::Create { json } => {
            let payload: CreateSetlist = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "POST",
                    "path": "api/v1/setlists",
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let setlist = client.create_setlist(payload).await?;
            output::print_json(&setlist, &output)
        }
        SetlistsCommand::Update { id, json } => {
            validate_resource_id(&id)?;
            let payload: CreateSetlist = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "PUT",
                    "path": format!("api/v1/setlists/{id}"),
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let setlist = client.update_setlist(&id, payload).await?;
            output::print_json(&setlist, &output)
        }
        SetlistsCommand::Patch { id, json } => {
            validate_resource_id(&id)?;
            let payload: serde_json::Value = serde_json::from_str(&json)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "PATCH",
                    "path": format!("api/v1/setlists/{id}"),
                    "body": payload,
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let setlist = client.patch_setlist(&id, payload).await?;
            output::print_json(&setlist, &output)
        }
        SetlistsCommand::Delete { id } => {
            validate_resource_id(&id)?;
            if dry_run {
                let planned = serde_json::json!({
                    "method": "DELETE",
                    "path": format!("api/v1/setlists/{id}"),
                });
                output::print_json(&planned, &output)?;
                return Ok(());
            }
            let setlist = client.delete_setlist(&id).await?;
            output::print_json(&setlist, &output)
        }
    }
}
