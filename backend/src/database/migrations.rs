use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result as AnyResult, anyhow};
use ring::digest::{SHA256, digest};
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;
use tracing::info;

#[derive(Debug, Deserialize, SurrealValue)]
struct AppliedMigration {
    script_name: String,
    checksum: String,
}

pub async fn run(db: &Surreal<Any>, migration_root: &str) -> AnyResult<()> {
    ensure_migration_table(db).await?;

    let migration_dir = resolve_migration_dir(migration_root)?;
    let files = list_migration_files(&migration_dir)?;
    let applied = load_applied_migrations(db).await?;

    for path in files {
        let script_name = file_name(&path)?;
        let script = fs::read_to_string(&path)
            .with_context(|| format!("failed to read migration script '{}'", path.display()))?;
        let checksum = script_checksum(&script);

        if let Some(existing_checksum) = applied.get(&script_name) {
            if existing_checksum != &checksum {
                return Err(anyhow!(
                    "migration '{}' checksum mismatch: expected {}, got {}",
                    script_name,
                    existing_checksum,
                    checksum
                ));
            }
            info!(
                migration = %script_name,
                status = "already_applied",
                "database migration already applied, skipping"
            );
            continue;
        }

        let started = Instant::now();
        info!(migration = %script_name, "applying database migration");
        apply_migration(db, &script_name, &checksum, &script).await?;
        let elapsed = started.elapsed();
        info!(
            migration = %script_name,
            duration_ms = elapsed.as_millis() as u64,
            status = "applied",
            "database migration finished successfully"
        );
    }

    Ok(())
}

async fn ensure_migration_table(db: &Surreal<Any>) -> AnyResult<()> {
    db.query(
        "DEFINE TABLE OVERWRITE migration_script TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE checksum ON migration_script TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE executed_at ON migration_script TYPE datetime READONLY VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD OVERWRITE script_name ON migration_script TYPE string PERMISSIONS FULL;
DEFINE INDEX OVERWRITE migration_script_script_name_unique ON migration_script FIELDS script_name UNIQUE CONCURRENTLY;",
    )
    .await
    .map_err(|err| anyhow!(err))
    .context("failed to define migration_script table")
    .map(|_| ())
}

async fn load_applied_migrations(db: &Surreal<Any>) -> AnyResult<HashMap<String, String>> {
    let mut response = db
        .query("SELECT script_name, checksum FROM migration_script;")
        .await
        .map_err(|err| anyhow!(err))
        .context("failed to read applied migration records")?;

    let rows: Vec<AppliedMigration> = response
        .take(0)
        .map_err(|err| anyhow!(err))
        .context("failed to decode applied migration records")?;

    let mut out = HashMap::with_capacity(rows.len());
    for row in rows {
        out.insert(row.script_name, row.checksum);
    }
    Ok(out)
}

fn ensure_no_statement_errors(
    migration: &str,
    context: &str,
    response: &mut surrealdb::IndexedResults,
) -> AnyResult<()> {
    let errors = response.take_errors();
    if errors.is_empty() {
        return Ok(());
    }

    let mut pairs: Vec<(usize, surrealdb::Error)> = errors.into_iter().collect();
    pairs.sort_by_key(|(idx, _)| *idx);

    let mut summary = Vec::with_capacity(pairs.len());
    for (idx, err) in &pairs {
        crate::observability::log_surreal_statement_error_migration(migration, *idx, err);
        summary.push(format!("[statement {idx}] {err}"));
    }

    Err(anyhow!("{}", summary.join("; "))).context(format!("{context} '{}'", migration))
}

async fn apply_migration(
    db: &Surreal<Any>,
    script_name: &str,
    checksum: &str,
    script: &str,
) -> AnyResult<()> {
    let tx = format!(
        "BEGIN TRANSACTION;
{};
COMMIT TRANSACTION;",
        script
    );

    let mut body_response = db
        .query(tx)
        .await
        .map_err(|err| anyhow!(err))
        .with_context(|| format!("failed to apply migration body '{}'", script_name))?;
    ensure_no_statement_errors(
        script_name,
        "migration body returned errors",
        &mut body_response,
    )?;
    body_response
        .check()
        .map_err(|err| anyhow!(err))
        .with_context(|| format!("migration body returned errors '{}'", script_name))?;

    let mut record_response = db
        .query("CREATE migration_script SET script_name = $script_name, checksum = $checksum;")
        .bind(("script_name", script_name.to_owned()))
        .bind(("checksum", checksum.to_owned()))
        .await
        .map_err(|err| anyhow!(err))
        .with_context(|| format!("failed to record migration '{}'", script_name))?;
    ensure_no_statement_errors(
        script_name,
        "record migration returned errors",
        &mut record_response,
    )?;
    record_response
        .check()
        .map_err(|err| anyhow!(err))
        .with_context(|| format!("record migration returned errors '{}'", script_name))?;

    Ok(())
}

fn resolve_migration_dir(migration_root: &str) -> AnyResult<PathBuf> {
    let root = resolve_absolute_path(migration_root)?;
    if !root.exists() {
        return Err(anyhow!(
            "migration directory '{}' does not exist",
            root.display()
        ));
    }
    if !root.is_dir() {
        return Err(anyhow!(
            "migration path '{}' is not a directory",
            root.display()
        ));
    }
    Ok(root)
}

fn resolve_absolute_path(path: &str) -> AnyResult<PathBuf> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Ok(candidate.to_path_buf());
    }

    let current_dir = env::current_dir().context("failed to resolve current working directory")?;
    Ok(current_dir.join(candidate))
}

fn list_migration_files(up_dir: &Path) -> AnyResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(up_dir)
        .with_context(|| format!("failed to read migration directory '{}'", up_dir.display()))?
    {
        let entry = entry.map_err(|err| anyhow!(err)).with_context(|| {
            format!(
                "failed to read entry in migration directory '{}'",
                up_dir.display()
            )
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("surql") {
            files.push(path);
        }
    }

    files.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    Ok(files)
}

fn file_name(path: &Path) -> AnyResult<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("invalid migration script file name: '{}'", path.display()))
}

fn script_checksum(script: &str) -> String {
    let digest = digest(&SHA256, script.as_bytes());
    hex::encode(digest.as_ref())
}
