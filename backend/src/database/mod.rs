mod migrations;

use anyhow::{Context, Result as AnyResult, anyhow};
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Database as DbAuth;

use crate::settings::Settings;

pub struct Database {
    pub db: Surreal<Any>,
}

impl Database {
    pub async fn new() -> AnyResult<Self> {
        let settings = Settings::global();

        let db = connect(&settings.db_address).await.with_context(|| {
            format!("failed to connect to SurrealDB at {}", settings.db_address)
        })?;

        match (&settings.db_username, &settings.db_password) {
            (Some(username), Some(password)) => {
                db.signin(DbAuth {
                    namespace: &settings.db_namespace,
                    database: &settings.db_database,
                    username,
                    password,
                })
                .await
                .with_context(|| "failed to sign into SurrealDB with provided credentials")?;
            }
            (None, None) => {}
            _ => {
                return Err(anyhow!(
                    "both DB username and password must be provided together"
                ));
            }
        }

        db.use_ns(&settings.db_namespace)
            .use_db(&settings.db_database)
            .await
            .with_context(|| {
                format!(
                    "failed to select SurrealDB namespace '{}' and database '{}'",
                    settings.db_namespace, settings.db_database
                )
            })?;

        Ok(Self { db })
    }

    pub async fn migrate(&self) -> AnyResult<()> {
        let settings = Settings::global();
        migrations::run(&self.db, &settings.db_migration_path).await
    }
}
