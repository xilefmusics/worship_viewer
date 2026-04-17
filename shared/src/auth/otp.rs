use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OtpRequest {
    pub email: String,
}

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OtpVerify {
    pub email: String,
    pub code: String,
}
