use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
#[allow(unused_imports)]
use serde_json::json;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({ "email": "singer@example.com" }))
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OtpRequest {
    pub email: String,
}

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[cfg_attr(
    feature = "backend",
    schema(example = json!({ "email": "singer@example.com", "code": "123456" }))
)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OtpVerify {
    pub email: String,
    pub code: String,
}
