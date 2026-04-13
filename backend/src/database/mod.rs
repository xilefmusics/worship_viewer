mod migrations;

use anyhow::{Context, Result as AnyResult, anyhow};
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::sql::{Id, Thing};

use crate::error::AppError;
use crate::settings::Settings;

#[derive(Deserialize)]
struct TeamIdOnly {
    id: Thing,
}

/// Stable string id for a `Thing` (matches API / legacy `id_to_plain_string` behavior).
pub(crate) fn record_id_string(thing: &Thing) -> String {
    match &thing.id {
        Id::String(value) => value.clone(),
        Id::Number(number) => format!("{number}"),
        Id::Uuid(uuid) => uuid.to_string(),
        _ => thing.id.to_string(),
    }
}

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

    /// The `team` row where `owner` is this user (their personal team).
    pub async fn personal_team_thing_for_user(&self, user_id: &str) -> Result<Thing, AppError> {
        let user = Thing::from(("user".to_owned(), user_id.to_owned()));
        let mut response = self
            .db
            .query("SELECT id FROM team WHERE owner = $user LIMIT 1")
            .bind(("user", user))
            .await
            .map_err(AppError::database)?;

        let rows: Vec<TeamIdOnly> = response.take(0).map_err(AppError::database)?;
        rows.into_iter()
            .next()
            .map(|r| r.id)
            .ok_or_else(|| AppError::database("personal team not found for user"))
    }
}
