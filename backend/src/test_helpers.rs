use anyhow::Result as AnyResult;

use crate::database::Database;
use crate::resources::{User, UserModel};

pub async fn test_db() -> AnyResult<Database> {
    let db = Database::connect("mem://", "test", "test", None, None).await?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/db-migrations");
    db.migrate(path).await?;
    Ok(db)
}

pub async fn seed_user(db: &Database) -> AnyResult<User> {
    Ok(db.create_user(User::new("smoke@test.local")).await?)
}
