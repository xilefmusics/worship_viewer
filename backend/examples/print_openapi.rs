//! Print the OpenAPI document as JSON to stdout (for CI / Spectral).

fn main() {
    let doc = backend::docs::openapi_document(&backend::settings::Settings::default());
    print!(
        "{}",
        serde_json::to_string_pretty(&serde_json::to_value(doc).expect("openapi as json"))
            .expect("pretty json")
    );
}
