use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Settings {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_namespace: String,
    pub db_database: String,
    pub host: String,
    pub port: u16,
    pub printer_host: String,
    pub printer_port: u16,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            db_host: std::env::var("DB_HOST").unwrap_or("localhost".into()),
            db_port: std::env::var("DB_PORT")
                .unwrap_or("8000".into())
                .parse::<u16>()
                .unwrap_or(8000),
            db_user: std::env::var("DB_USER").unwrap_or("root".into()),
            db_password: std::env::var("DB_PASSWORD").unwrap_or("root".into()),
            db_namespace: std::env::var("DB_NAMESPACE").unwrap_or("test".into()),
            db_database: std::env::var("DB_DATABASE").unwrap_or("test".into()),
            host: std::env::var("HOST").unwrap_or("0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or("8082".into())
                .parse::<u16>()
                .unwrap_or(8082),
            printer_host: std::env::var("PRINTER_HOST").unwrap_or("localhost".into()),
            printer_port: std::env::var("PRINTER_PORT")
                .unwrap_or("3000".into())
                .parse::<u16>()
                .unwrap_or(3000),
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn get() -> &'static Settings {
    SETTINGS.get_or_init(Settings::new)
}