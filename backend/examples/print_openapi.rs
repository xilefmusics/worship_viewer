//! Print the OpenAPI document as JSON to stdout (for CI / Spectral).
use utoipa::OpenApi;

fn main() {
    let doc = backend::docs::ApiDoc::openapi();
    print!(
        "{}",
        serde_json::to_string_pretty(&serde_json::to_value(doc).expect("openapi as json"))
            .expect("pretty json")
    );
}
