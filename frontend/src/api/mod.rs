mod api;
pub use api::Api;

mod provider;
pub use provider::{use_api, ApiProvider};

mod types;
pub use types::{CreateUserRequest, ErrorResponse, OtpRequestPayload, OtpVerifyPayload, Session, User};

mod error;
pub use error::ApiError;
