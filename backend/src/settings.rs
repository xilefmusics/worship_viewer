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
}

#[derive(Clone, Debug)]
pub struct PrinterConfig {
    pub address: String,
    pub api_key: String,
}

#[derive(Deserialize, Debug)]
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

    #[serde(default)]
    pub apple_issuer_url: Option<String>,
    #[serde(default)]
    pub apple_client_id: Option<String>,
    #[serde(default)]
    pub apple_client_secret: Option<String>,
    #[serde(default)]
    pub apple_redirect_url: Option<String>,
    #[serde(default)]
    pub apple_scopes: Option<Vec<String>>,

    pub initial_admin_user_email: Option<String>,
    pub initial_admin_user_test_session: bool,

    pub gmail_app_password: String,
    pub gmail_from: String,

    pub static_dir: String,
    pub blob_dir: String,

    pub printer_address: String,
    pub printer_api_key: String,
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
            apple_issuer_url: None,
            apple_client_id: None,
            apple_client_secret: None,
            apple_redirect_url: None,
            apple_scopes: None,
            initial_admin_user_email: None,
            initial_admin_user_test_session: false,
            gmail_app_password: String::new(),
            gmail_from: String::new(),
            static_dir: "static".into(),
            blob_dir: "blobs".into(),
            printer_address: "http://localhost:3000".into(),
            printer_api_key: "changeme".into(),
        }
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
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
        }
    }

    pub fn printer_config(&self) -> PrinterConfig {
        PrinterConfig {
            address: self.printer_address.clone(),
            api_key: self.printer_api_key.clone(),
        }
    }
}
