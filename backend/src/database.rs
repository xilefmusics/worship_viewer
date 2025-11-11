use anyhow::{Context, Result as AnyResult, anyhow};
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb_migrations::MigrationRunner;

use std::env;
use std::path::Path;

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

        let migration_path = resolve_migration_path(&settings.db_migration_path)?;
        unsafe {
            env::set_var("SURREAL_MIG_PATH", &migration_path);
        }

        MigrationRunner::new(&self.db)
            .up()
            .await
            .map_err(|err| anyhow!(err))
            .context("failed to apply SurrealDB migrations")?;
        Ok(())
    }
}

fn resolve_migration_path(path: &str) -> AnyResult<String> {
    let candidate = Path::new(path);
    let absolute = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        env::current_dir()
            .context("failed to resolve current working directory")?
            .join(candidate)
    };

    let absolute_str = absolute
        .to_str()
        .ok_or_else(|| anyhow!("migration path is not valid UTF-8"))?;

    Ok(absolute_str.to_owned())
}
