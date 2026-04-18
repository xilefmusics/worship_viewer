mod migrations;

use anyhow::{Context, Result as AnyResult, anyhow};
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::sql::{Id, Thing};
use tracing::instrument;

use crate::error::AppError;

/// Inspect Surreal [`surrealdb::Response`] for per-statement failures (mirrors migration checks).
pub(crate) fn surreal_take_errors(
    context: &'static str,
    response: &mut surrealdb::Response,
) -> Result<(), AppError> {
    let errors = response.take_errors();
    if errors.is_empty() {
        return Ok(());
    }

    let mut pairs: Vec<(usize, surrealdb::Error)> = errors.into_iter().collect();
    pairs.sort_by_key(|(idx, _)| *idx);

    let mut summary = Vec::with_capacity(pairs.len());
    for (idx, err) in pairs {
        tracing::error!(
            context = context,
            statement_index = idx,
            error = %err,
            error_source_chain = %crate::observability::error_source_chain_string(&err),
            error_debug = ?err,
            "SurrealDB query statement failed"
        );
        summary.push(format!("[statement {idx}] {err}"));
    }

    Err(AppError::database(format!(
        "{context}: {}",
        summary.join("; ")
    )))
}

#[derive(Deserialize)]
struct TeamIdOnly {
    id: Thing,
}

/// Stable string id for a `Thing` (matches API / legacy `id_to_plain_string` behavior).
pub fn record_id_string(thing: &Thing) -> String {
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
    #[instrument(
        name = "database.connect",
        level = "debug",
        err,
        skip(username, password),
        fields(
            db_address = %address,
            db_namespace = %namespace,
            db_database = %database,
            has_credentials = username.is_some() && password.is_some()
        )
    )]
    pub async fn connect(
        address: &str,
        namespace: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> AnyResult<Self> {
        let db = connect(address)
            .await
            .with_context(|| format!("failed to connect to SurrealDB at {address}"))?;

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

    #[instrument(
        name = "database.migrate",
        level = "debug",
        err,
        skip(self),
        fields(migration_path = %migration_path)
    )]
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
            .map_err(|e| {
                crate::log_and_convert!(AppError::database, "db.personal_team.query", e)
            })?;

        surreal_take_errors("db.personal_team", &mut response)?;
        response = response.check().map_err(|e| {
            crate::log_and_convert!(AppError::database, "db.personal_team.check", e)
        })?;

        let rows: Vec<TeamIdOnly> = response
            .take(0)
            .map_err(|e| crate::log_and_convert!(AppError::database, "db.personal_team.take", e))?;
        rows.into_iter()
            .next()
            .map(|r| r.id)
            .ok_or_else(|| AppError::database("personal team not found for user"))
    }
}
