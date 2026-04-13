use std::collections::BTreeSet;

use actix_web::web::Data;
use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::user::{Role as UserRole, User};

use crate::database::Database;
use crate::error::AppError;

use super::model::{
    TeamFetched, can_read_team, public_team_thing, team_content_writable, team_fetched_to_stored,
    thing_record_key,
};

/// Resolves which team [`Thing`]s apply for content ACL (read vs write).
#[async_trait]
pub trait TeamResolver: Send + Sync {
    async fn content_read_teams(&self, user: &User) -> Result<Vec<Thing>, AppError>;
    async fn content_write_teams(&self, user: &User) -> Result<Vec<Thing>, AppError>;
    async fn personal_team(&self, user_id: &str) -> Result<Thing, AppError>;
}

/// Production resolver backed by [`Database`].
#[derive(Clone)]
pub struct SurrealTeamResolver {
    db: Data<Database>,
}

impl SurrealTeamResolver {
    pub fn new(db: Data<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TeamResolver for SurrealTeamResolver {
    async fn content_read_teams(&self, user: &User) -> Result<Vec<Thing>, AppError> {
        content_read_team_things(self.db.get_ref(), user).await
    }

    async fn content_write_teams(&self, user: &User) -> Result<Vec<Thing>, AppError> {
        content_write_team_things(self.db.get_ref(), user).await
    }

    async fn personal_team(&self, user_id: &str) -> Result<Thing, AppError> {
        self.db
            .get_ref()
            .personal_team_thing_for_user(user_id)
            .await
    }
}

/// Teams whose content the user may list/read (GET), including `team:public` for catalog.
pub async fn content_read_team_things(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
    let app_admin = user.role == UserRole::Admin;
    let public_thing = public_team_thing();
    let rows = db
        .db
        .query("SELECT * FROM team WHERE id != $public FETCH owner, members.user")
        .bind(("public", public_thing.clone()))
        .await?
        .take::<Vec<TeamFetched>>(0)?;

    let mut out: Vec<Thing> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    let mut push = |t: Thing| {
        let key = thing_record_key(&t);
        if seen.insert(key) {
            out.push(t);
        }
    };

    push(public_thing);

    for row in rows {
        let stored = team_fetched_to_stored(&row)?;
        if can_read_team(&user.id, &stored, app_admin) {
            push(row.id.clone());
        }
    }

    Ok(out)
}

/// Teams whose content the user may create/update/delete. Platform admin does not imply global write.
pub async fn content_write_team_things(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
    let public_thing = public_team_thing();
    let rows = db
        .db
        .query("SELECT * FROM team WHERE id != $public FETCH owner, members.user")
        .bind(("public", public_thing))
        .await?
        .take::<Vec<TeamFetched>>(0)?;

    let mut out: Vec<Thing> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for row in rows {
        let stored = team_fetched_to_stored(&row)?;
        if team_content_writable(&user.id, &stored) {
            let key = thing_record_key(&row.id);
            if seen.insert(key) {
                out.push(row.id.clone());
            }
        }
    }

    Ok(out)
}
