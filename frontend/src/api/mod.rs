mod api;
pub use api::Api;

mod provider;
pub use provider::{use_api, ApiProvider};

#[allow(unused_imports)]
pub use shared::auth::otp::{OtpRequest, OtpVerify};
#[allow(unused_imports)]
pub use shared::error::ErrorResponse;
#[allow(unused_imports)]
pub use shared::user::{CreateUserRequest, Session, User};

mod error;
pub use error::ApiError;
