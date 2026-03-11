use serde_json::Value;

use shared::api::ApiClient;
use shared::net::DefaultHttpClient;

use crate::output::{self, OutputFormat};

pub async fn handle_schema(
    client: &ApiClient<DefaultHttpClient>,
    output: OutputFormat,
    path_prefix: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut schema: Value = client.get_openapi_docs().await?;

    if let Some(prefix) = path_prefix {
        if let Some(paths) = schema.get_mut("paths").and_then(|v| v.as_object_mut()) {
            paths.retain(|k, _| k.starts_with(&prefix));
        }
    }

    output::print_json(&schema, &output)
}
