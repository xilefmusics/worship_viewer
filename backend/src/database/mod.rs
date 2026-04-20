mod migrations;

use anyhow::{Context, Result as AnyResult, anyhow};
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::types::{RecordId, RecordIdKey, SqlFormat, SurrealValue, ToSql};
use tracing::instrument;

use crate::error::AppError;

/// Inspect Surreal [`surrealdb::IndexedResults`] for per-statement failures (mirrors migration checks).
pub(crate) fn surreal_take_errors(
    context: &'static str,
    response: &mut surrealdb::IndexedResults,
) -> Result<(), AppError> {
    let errors = response.take_errors();
    if errors.is_empty() {
        return Ok(());
    }

    let mut pairs: Vec<(usize, surrealdb::Error)> = errors.into_iter().collect();
    pairs.sort_by_key(|(idx, _)| *idx);

    let mut summary = Vec::with_capacity(pairs.len());
    for (idx, err) in pairs {
        crate::observability::log_surreal_statement_error_context(context, idx, &err);
        summary.push(format!("[statement {idx}] {err}"));
    }

    Err(AppError::database(format!(
        "{context}: {}",
        summary.join("; ")
    )))
}

#[derive(Debug, Deserialize, SurrealValue)]
struct TeamIdOnly {
    id: RecordId,
}

/// Stable string id for the key portion of a [`RecordId`] (matches API / legacy `id_to_plain_string` behavior).
pub fn record_id_string(rid: &RecordId) -> String {
    match &rid.key {
        RecordIdKey::String(value) => value.clone(),
        RecordIdKey::Number(number) => format!("{number}"),
        RecordIdKey::Uuid(uuid) => uuid.to_string(),
        _ => {
            let mut out = String::new();
            rid.key.fmt_sql(&mut out, SqlFormat::SingleLine);
            out
        }
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
                    namespace: namespace.to_owned(),
                    database: database.to_owned(),
                    username: username.to_owned(),
                    password: password.to_owned(),
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
    pub async fn personal_team_thing_for_user(&self, user_id: &str) -> Result<RecordId, AppError> {
        let user = RecordId::new("user", user_id.to_owned());
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
