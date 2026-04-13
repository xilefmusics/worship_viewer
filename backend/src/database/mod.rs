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
    /// Production constructor — reads connection settings from global [`Settings`].
    pub async fn new() -> AnyResult<Self> {
        let settings = Settings::global();
        Self::connect(
            &settings.db_address,
            &settings.db_namespace,
            &settings.db_database,
            settings.db_username.as_deref(),
            settings.db_password.as_deref(),
        )
        .await
    }

    /// Explicit constructor — used by tests and any caller that must not depend on [`Settings::global`].
    pub async fn connect(
        address: &str,
        namespace: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> AnyResult<Self> {
        let db = connect(address).await.with_context(|| {
            format!("failed to connect to SurrealDB at {address}")
        })?;

        match (username, password) {
            (Some(username), Some(password)) => {
                db.signin(DbAuth {
                    namespace,
                    database,
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

        db.use_ns(namespace)
            .use_db(database)
            .await
            .with_context(|| {
                format!(
                    "failed to select SurrealDB namespace '{namespace}' and database '{database}'"
                )
            })?;

        Ok(Self { db })
    }

    pub async fn migrate(&self, migration_path: &str) -> AnyResult<()> {
        migrations::run(&self.db, migration_path).await
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
