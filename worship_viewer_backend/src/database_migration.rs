use super::database::Database;
use super::error::AppError;
use super::types::Collection;
use log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Version {
    pub version: usize,
}

async fn get_current_version(database: &Database) -> usize {
    database
        .query_one("SELECT version FROM version:version;".into())
        .await
        .unwrap_or(Version { version: 0 })
        .version
}

async fn migrate_0_1(database: &Database) -> Result<(), AppError> {
    log::info!("Migrate the Database from version 0 to version 1");
    let sql = include_str!("./database_migration_sql/migrate_0_1.sql");
    let result = database.query_string(sql.into()).await?;
    log::info!("Result: {}", result);
    Ok(())
}

async fn migrate_1_2(database: &Database) -> Result<(), AppError> {
    log::info!("Migrate the Database from version 1 to version 2");
    let nr_sql = database
        .query_vec::<Vec<Collection>>("Select * FROM collection;".into())
        .await?
        .into_iter()
        .map(|collection| {
            collection.songs.into_iter().map(|song| {
                format!(
                    "UPDATE {} SET nr = \"{}\";",
                    song,
                    song.split("song").skip(2).next().unwrap_or(""),
                )
            })
        })
        .flatten()
        .collect::<String>();
    let sql = include_str!("./database_migration_sql/migrate_1_2.sql")
        .replace("/* REPLACE nr_sql */", &nr_sql);
    let result = database.query_string(sql.into()).await?;
    log::info!("Result: {}", result);
    Ok(())
}

pub async fn migrate(database: &Database) -> Result<(), AppError> {
    let exspected_version = 2;
    let current_version = get_current_version(database).await;

    if current_version > exspected_version {
        return Err(AppError::DatabaseMigration(
            "The database is newer as exspected!".into(),
        ));
    }

    if current_version == exspected_version {
        log::info!("The database version is up to date");
        return Ok(());
    }

    log::info!(
        "The database is on version {}, but the code exspects it to be on version {}",
        current_version,
        exspected_version,
    );

    if current_version == 0 {
        migrate_0_1(database).await?
    }

    if current_version == 1 {
        migrate_1_2(database).await?
    }

    Ok(())
}
