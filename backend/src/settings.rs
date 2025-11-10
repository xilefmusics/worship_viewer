use once_cell::sync::OnceCell;
use serde::Deserialize;

static SETTINGS: OnceCell<Settings> = OnceCell::new();

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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            post_login_path: "/".into(),
            cookie_name: "sso_session".into(),
            cookie_secure: false,
            session_ttl_seconds: 3600,
            otp_ttl_seconds: 300,
            otp_pepper: "changeme".into(),
            db_address: "mem://".into(),
            db_namespace: "app".into(),
            db_database: "app".into(),
            db_username: None,
            db_password: None,
            db_migration_path: "surrealdb".into(),
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
        }
    }
}

impl Settings {
    fn read() -> Result<Self, envy::Error> {
        let settings = envy::from_env::<Self>()?;
        Ok(settings)
    }

    pub fn init() -> Result<&'static Self, envy::Error> {
        SETTINGS.get_or_try_init(Self::read)
    }

    pub fn global() -> &'static Self {
        SETTINGS
            .get()
            .expect("Settings::global called before initialization")
    }
}
