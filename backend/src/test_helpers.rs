use std::sync::Arc;

use anyhow::Result as AnyResult;

use crate::database::Database;
use crate::resources::{User, UserModel};

pub async fn test_db() -> AnyResult<Arc<Database>> {
    let db = Database::connect("mem://", "test", "test", None, None).await?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/db-migrations");
    db.migrate(path).await?;
    Ok(Arc::new(db))
}

pub async fn seed_user(db: &Arc<Database>) -> AnyResult<User> {
    Ok(db.create_user(User::new("smoke@test.local")).await?)
}
