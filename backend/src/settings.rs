use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct CookieConfig {
    pub name: String,
    pub secure: bool,
    pub session_ttl_seconds: u64,
    pub post_login_path: String,
}

#[derive(Clone, Debug)]
pub struct OtpConfig {
    pub ttl_seconds: u64,
    pub pepper: String,
    pub max_attempts: u32,
    /// When false, OTP verify rejects unknown emails instead of creating a user.
    pub allow_self_signup: bool,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub post_login_path: String,
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub session_ttl_seconds: u64,

    pub otp_ttl_seconds: u64,
    pub otp_pepper: String,
    pub otp_max_attempts: u32,
    /// When false, `/auth/otp/verify` requires an existing user (no implicit signup). Default true.
    #[serde(default = "default_otp_allow_self_signup")]
    pub otp_allow_self_signup: bool,

    pub db_address: String,
    pub db_namespace: String,
    pub db_database: String,
    pub db_username: Option<String>,
    pub db_password: Option<String>,
    pub db_migration_path: String,

    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: Option<String>,
    pub oidc_redirect_url: String,
    pub oidc_scopes: Vec<String>,

    pub initial_admin_user_email: Option<String>,
    pub initial_admin_user_test_session: bool,

    pub gmail_app_password: String,
    pub gmail_from: String,

    pub static_dir: String,
    pub blob_dir: String,
    /// Maximum allowed size (in bytes) for binary blob uploads via `PUT /blobs/{id}/data`.
    /// Default: 20 MiB.
    pub blob_upload_max_bytes: usize,

    /// Requests per second allowed per IP on sensitive auth endpoints (OTP + login).
    /// Default: 1 request per second with a burst of 5.
    pub auth_rate_limit_rps: u64,
    pub auth_rate_limit_burst: u32,

    /// Per-IP rate limit for `/api/v1/*` (token bucket). Defaults are generous for local development.
    pub api_rate_limit_rps: u64,
    pub api_rate_limit_burst: u32,

    /// Shown under `info.contact.email` in OpenAPI when set (`OPENAPI_CONTACT_EMAIL`).
    #[serde(default)]
    pub openapi_contact_email: Option<String>,
    /// Legal imprint / contact page URL under `info.contact.url` when set (`OPENAPI_IMPRINT_URL`).
    #[serde(default)]
    pub openapi_imprint_url: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            post_login_path: "/".into(),
            cookie_name: "sso_session".into(),
            cookie_secure: false,
            session_ttl_seconds: 31536000,
            otp_ttl_seconds: 300,
            otp_pepper: "changeme".into(),
            otp_max_attempts: 5,
            otp_allow_self_signup: true,
            db_address: "mem://".into(),
            db_namespace: "app".into(),
            db_database: "app".into(),
            db_username: None,
            db_password: None,
            db_migration_path: "db-migrations".into(),
            oidc_issuer_url: "https://accounts.google.com".into(),
            oidc_client_id: String::new(),
            oidc_client_secret: None,
            oidc_redirect_url: "http://localhost:8080/auth/callback".into(),
            oidc_scopes: vec!["openid".into(), "profile".into(), "email".into()],
            initial_admin_user_email: None,
            initial_admin_user_test_session: false,
            gmail_app_password: String::new(),
            gmail_from: String::new(),
            static_dir: "static".into(),
            blob_dir: "blobs".into(),
            blob_upload_max_bytes: 20 * 1024 * 1024,
            auth_rate_limit_rps: 1,
            auth_rate_limit_burst: 5,
            api_rate_limit_rps: 50,
            api_rate_limit_burst: 200,
            openapi_contact_email: None,
            openapi_imprint_url: None,
        }
    }
}

fn default_otp_allow_self_signup() -> bool {
    true
}

impl Settings {
    pub fn from_env() -> Result<Self, envy::Error> {
        let mut s = envy::from_env::<Self>()?;
        if let Ok(v) = std::env::var("WORSHIP_OTP_ALLOW_SELF_SIGNUP") {
            s.otp_allow_self_signup =
                !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("no"));
        }
        Ok(s)
    }

    pub fn cookie_config(&self) -> CookieConfig {
        CookieConfig {
            name: self.cookie_name.clone(),
            secure: self.cookie_secure,
            session_ttl_seconds: self.session_ttl_seconds,
            post_login_path: self.post_login_path.clone(),
        }
    }

    pub fn otp_config(&self) -> OtpConfig {
        OtpConfig {
            ttl_seconds: self.otp_ttl_seconds,
            pepper: self.otp_pepper.clone(),
            max_attempts: self.otp_max_attempts,
            allow_self_signup: self.otp_allow_self_signup,
        }
    }
}
