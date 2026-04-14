use std::collections::BTreeSet;

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::sql::Thing;
use tokio::sync::OnceCell;

use shared::user::{Role as UserRole, User};

use crate::database::Database;
use crate::error::AppError;

use super::model::{public_team_thing, thing_record_key, user_thing};

#[derive(Debug, Deserialize)]
struct TeamIdRow {
    id: Thing,
}

/// Resolves which team [`Thing`]s apply for content ACL (read vs write).
#[async_trait]
pub trait TeamResolver: Send + Sync {
    async fn content_read_teams(&self, user: &User) -> Result<Vec<Thing>, AppError>;
    async fn content_write_teams(&self, user: &User) -> Result<Vec<Thing>, AppError>;
    async fn personal_team(&self, user_id: &str) -> Result<Thing, AppError>;
}

/// Per-request caching wrapper around a [`User`] and a [`TeamResolver`].
///
/// Team lists are resolved lazily on first access and cached for the lifetime of
/// this struct. Construct one at the start of a request handler and pass it into
/// service methods instead of a bare `&User`.
pub struct UserPermissions<'a, T: TeamResolver> {
    user: &'a User,
    resolver: &'a T,
    read_teams: OnceCell<Vec<Thing>>,
    write_teams: OnceCell<Vec<Thing>>,
    personal_team: OnceCell<Thing>,
}

impl<'a, T: TeamResolver> UserPermissions<'a, T> {
    pub fn new(user: &'a User, resolver: &'a T) -> Self {
        Self {
            user,
            resolver,
            read_teams: OnceCell::new(),
            write_teams: OnceCell::new(),
            personal_team: OnceCell::new(),
        }
    }

    pub fn user(&self) -> &User {
        self.user
    }

    /// Teams whose content the user may read. Resolved once; subsequent calls return the cached slice.
    pub async fn read_teams(&self) -> Result<&[Thing], AppError> {
        let user = self.user;
        let resolver = self.resolver;
        self.read_teams
            .get_or_try_init(|| async move { resolver.content_read_teams(user).await })
            .await
            .map(Vec::as_slice)
    }

    /// Teams whose content the user may write. Resolved once; subsequent calls return the cached slice.
    pub async fn write_teams(&self) -> Result<&[Thing], AppError> {
        let user = self.user;
        let resolver = self.resolver;
        self.write_teams
            .get_or_try_init(|| async move { resolver.content_write_teams(user).await })
            .await
            .map(Vec::as_slice)
    }

    /// The user's personal team. Resolved once; subsequent calls return a clone.
    pub async fn personal_team(&self) -> Result<Thing, AppError> {
        let user_id = self.user.id.clone();
        let resolver = self.resolver;
        self.personal_team
            .get_or_try_init(|| async move { resolver.personal_team(&user_id).await })
            .await
            .map(Thing::clone)
    }
}

/// Production resolver backed by [`Database`].
#[derive(Clone)]
pub struct SurrealTeamResolver {
    db: Arc<Database>,
}

impl SurrealTeamResolver {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TeamResolver for SurrealTeamResolver {
    async fn content_read_teams(&self, user: &User) -> Result<Vec<Thing>, AppError> {
        content_read_team_things(&self.db, user).await
    }

    async fn content_write_teams(&self, user: &User) -> Result<Vec<Thing>, AppError> {
        content_write_team_things(&self.db, user).await
    }

    async fn personal_team(&self, user_id: &str) -> Result<Thing, AppError> {
        self.db.personal_team_thing_for_user(user_id).await
    }
}

/// Teams whose content the user may list/read (GET), including `team:public` for catalog.
pub async fn content_read_team_things(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
    let app_admin = user.role == UserRole::Admin;
    let public_thing = public_team_thing();
    let ut = user_thing(&user.id);

    let mut out: Vec<Thing> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    let mut push = |t: Thing| {
        let key = thing_record_key(&t);
        if seen.insert(key) {
            out.push(t);
        }
    };

    push(public_thing.clone());

    let rows: Vec<TeamIdRow> = if app_admin {
        db.db
            .query("SELECT id FROM team WHERE id != $public")
            .bind(("public", public_thing.clone()))
            .await?
            .take(0)?
    } else {
        db.db
            .query(
                "SELECT id FROM team WHERE id != $public AND (owner = $user \
                 OR array::len(members[WHERE user = $user]) > 0)",
            )
            .bind(("public", public_thing.clone()))
            .bind(("user", ut))
            .await?
            .take(0)?
    };

    for row in rows {
        push(row.id);
    }

    Ok(out)
}

/// Teams whose content the user may create/update/delete. Platform admin does not imply global write.
pub async fn content_write_team_things(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
    let public_thing = public_team_thing();
    let ut = user_thing(&user.id);

    let rows: Vec<TeamIdRow> = db
        .db
        .query(
            "SELECT id FROM team WHERE id != $public AND (owner = $user \
             OR array::len(members[WHERE user = $user AND (role = 'admin' \
             OR role = 'content_maintainer')]) > 0)",
        )
        .bind(("public", public_thing))
        .bind(("user", ut))
        .await?
        .take(0)?;

    let mut out: Vec<Thing> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for row in rows {
        let key = thing_record_key(&row.id);
        if seen.insert(key) {
            out.push(row.id);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{seed_user, test_db};

    use super::super::model::{
        TeamFetched, can_read_team, team_content_writable, team_fetched_to_stored,
    };

    fn thing_key_set(things: &[Thing]) -> BTreeSet<String> {
        things.iter().map(|t| thing_record_key(t)).collect()
    }

    async fn naive_read_teams(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
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

    async fn naive_write_teams(db: &Database, user: &User) -> Result<Vec<Thing>, AppError> {
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

    #[tokio::test]
    async fn content_read_teams_matches_naive_rust_filter() {
        let db = test_db().await.expect("test db");
        let user = seed_user(&db).await.expect("user");
        let dbref: &Database = db.as_ref();
        let a = content_read_team_things(dbref, &user).await.expect("sql read");
        let b = naive_read_teams(dbref, &user).await.expect("rust read");
        assert_eq!(thing_key_set(&a), thing_key_set(&b));
    }

    #[tokio::test]
    async fn content_write_teams_matches_naive_rust_filter() {
        let db = test_db().await.expect("test db");
        let user = seed_user(&db).await.expect("user");
        let dbref: &Database = db.as_ref();
        let a = content_write_team_things(dbref, &user).await.expect("sql write");
        let b = naive_write_teams(dbref, &user).await.expect("rust write");
        assert_eq!(thing_key_set(&a), thing_key_set(&b));
    }

    #[tokio::test]
    async fn content_read_teams_matches_for_app_admin() {
        let db = test_db().await.expect("test db");
        let mut user = seed_user(&db).await.expect("user");
        user.role = UserRole::Admin;
        let dbref: &Database = db.as_ref();
        let a = content_read_team_things(dbref, &user).await.expect("sql read");
        let b = naive_read_teams(dbref, &user).await.expect("rust read");
        assert_eq!(thing_key_set(&a), thing_key_set(&b));
    }
}
