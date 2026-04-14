use std::collections::BTreeMap;

use std::sync::Arc;

use shared::team::{CreateTeam, Team, UpdateTeam};
use shared::user::{Role as UserRole, User};

use crate::database::Database;
use crate::error::AppError;

use super::model::{
    TeamCreatePayload, build_create_shared_members, can_read_team, effective_admin,
    ensure_shared_team_has_admin_after_update, inputs_to_db_members,
    member_or_owner_readable, member_self_leave_payload, team_fetched_to_stored,
    team_resource_or_reject_public, thing_user_id, validate_personal_members_not_owner,
};
use super::repository::TeamRepository;
use super::resolver::{TeamResolver, UserPermissions};
use super::surreal_repo::SurrealTeamRepo;

/// Application service: authorization and orchestration for teams.
#[derive(Clone)]
pub struct TeamService<R, TR> {
    pub repo: R,
    pub resolver: TR,
}

impl<R, TR> TeamService<R, TR> {
    pub fn new(repo: R, resolver: TR) -> Self {
        Self { repo, resolver }
    }
}

impl<R: TeamRepository, TR: TeamResolver> TeamService<R, TR> {
    pub async fn list_teams_for_user(&self, user: &User) -> Result<Vec<Team>, AppError> {
        let app_admin = user.role == UserRole::Admin;
        let rows = self.repo.fetch_teams_for_user(&user.id, app_admin).await?;
        let mut by_id: BTreeMap<String, Team> = BTreeMap::new();
        for row in rows {
            let team = row.into_team()?;
            by_id.insert(team.id.clone(), team);
        }
        let mut list: Vec<Team> = by_id.into_values().collect();
        list.sort_by(|a, b| match (a.owner.is_some(), b.owner.is_some()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.id.cmp(&b.id),
        });
        Ok(list)
    }

    pub async fn get_team_for_user(&self, user: &User, id: &str) -> Result<Team, AppError> {
        let app_admin = user.role == UserRole::Admin;
        let row = self
            .repo
            .fetch_team(id)
            .await?
            .ok_or_else(|| AppError::NotFound("team not found".into()))?;
        let stored = team_fetched_to_stored(&row)?;
        if !can_read_team(&user.id, &stored, app_admin) {
            return Err(AppError::NotFound("team not found".into()));
        }
        row.into_team()
    }

    pub async fn create_shared_team_for_user(
        &self,
        user: &User,
        payload: CreateTeam,
    ) -> Result<Team, AppError> {
        let name = payload.name.trim().to_owned();
        if name.is_empty() {
            return Err(AppError::invalid_request("team name must not be empty"));
        }
        let members = build_create_shared_members(&user.id, &payload.members)?;
        let id = self
            .repo
            .create_team(TeamCreatePayload { name, owner: None, members })
            .await?;
        self.repo.load_team_display(&id).await
    }

    pub async fn update_team_for_user(
        &self,
        user: &User,
        id: &str,
        payload: UpdateTeam,
    ) -> Result<Team, AppError> {
        let resource = team_resource_or_reject_public(id)?;
        let name_trim = payload.name.trim().to_owned();
        if name_trim.is_empty() {
            return Err(AppError::invalid_request("team name must not be empty"));
        }

        let row = self
            .repo
            .fetch_team(id)
            .await?
            .ok_or_else(|| AppError::NotFound("team not found".into()))?;

        let current_name = row.name.trim().to_owned();
        let stored = team_fetched_to_stored(&row)?;
        if !member_or_owner_readable(&user.id, &stored) {
            return Err(AppError::NotFound("team not found".into()));
        }

        let admin = effective_admin(&user.id, &stored);

        if !admin {
            let Some(ref inputs) = payload.members else {
                return Err(AppError::forbidden());
            };
            let new_members = inputs_to_db_members(inputs)?;
            if !member_self_leave_payload(
                &stored,
                &user.id,
                &current_name,
                &name_trim,
                &new_members,
            ) {
                return Err(AppError::forbidden());
            }
            if stored.owner.is_some() {
                let owner_id = stored
                    .owner
                    .as_ref()
                    .map(thing_user_id)
                    .ok_or_else(|| AppError::database("personal team missing owner"))?;
                validate_personal_members_not_owner(&owner_id, &new_members)?;
            } else {
                ensure_shared_team_has_admin_after_update(&new_members)?;
            }
            self.repo
                .update_team_members(resource, new_members)
                .await?;
            return self.repo.load_team_display(id).await;
        }

        self.repo
            .update_team_name(resource.clone(), &name_trim)
            .await?;

        if let Some(inputs) = payload.members {
            let new_members = inputs_to_db_members(&inputs)?;
            if stored.owner.is_some() {
                let owner_id = stored
                    .owner
                    .as_ref()
                    .map(thing_user_id)
                    .ok_or_else(|| AppError::database("personal team missing owner"))?;
                validate_personal_members_not_owner(&owner_id, &new_members)?;
            } else {
                ensure_shared_team_has_admin_after_update(&new_members)?;
            }
            self.repo.update_team_members(resource, new_members).await?;
        }

        self.repo.load_team_display(id).await
    }

    pub async fn delete_team_for_user(
        &self,
        perms: &UserPermissions<'_, TR>,
        id: &str,
    ) -> Result<Team, AppError> {
        let resource = team_resource_or_reject_public(id)?;

        let row = self
            .repo
            .fetch_team(id)
            .await?
            .ok_or_else(|| AppError::NotFound("team not found".into()))?;

        let stored = team_fetched_to_stored(&row)?;
        if stored.owner.is_some() {
            return Err(AppError::forbidden());
        }
        if !effective_admin(&perms.user().id, &stored) {
            return Err(AppError::forbidden());
        }

        let team = row.into_team()?;
        let personal = perms.personal_team().await?;
        let from = surrealdb::sql::Thing::from(resource.clone());
        self.repo.reassign_content(from, personal).await?;
        self.repo.delete_team_record(resource).await?;

        Ok(team)
    }
}

/// Production type alias used in HTTP wiring.
pub type TeamServiceHandle = TeamService<SurrealTeamRepo, super::resolver::SurrealTeamResolver>;

impl TeamServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        TeamService::new(
            SurrealTeamRepo::new(db.clone()),
            super::resolver::SurrealTeamResolver::new(db.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use shared::team::CreateTeam;

    use crate::error::AppError;
    use crate::resources::team::UserPermissions;
    use crate::test_helpers::{create_user, test_db};

    use super::*;

    fn team_service(db: &std::sync::Arc<crate::database::Database>) -> TeamServiceHandle {
        crate::test_helpers::team_service(db)
    }

    #[tokio::test]
    async fn blc_team_shared_create_and_list() {
        let db = test_db().await.expect("db");
        let u = create_user(&db, "team-creator@test.local").await.expect("u");
        let svc = team_service(&db);
        let t = svc
            .create_shared_team_for_user(
                &u,
                CreateTeam { name: "Band".into(), members: vec![] },
            )
            .await
            .expect("shared team");

        assert!(!t.id.is_empty());
        assert_eq!(t.name, "Band");

        let teams = svc.list_teams_for_user(&u).await.expect("teams");
        assert!(teams.iter().any(|x| x.id == t.id));
    }

    #[tokio::test]
    async fn blc_team_personal_cannot_delete() {
        let db = test_db().await.expect("db");
        let u = create_user(&db, "team-personal@test.local").await.expect("u");
        let svc = team_service(&db);
        let teams = svc.list_teams_for_user(&u).await.expect("teams");
        let personal = teams
            .iter()
            .find(|t| t.owner.as_ref().map(|o| o.id == u.id).unwrap_or(false))
            .expect("personal");
        let perms = UserPermissions::new(&u, &svc.resolver);
        let err = svc.delete_team_for_user(&perms, &personal.id).await;
        assert!(matches!(err, Err(AppError::Forbidden)));
    }

    #[tokio::test]
    async fn blc_team_delete_shared_empty_team() {
        let db = test_db().await.expect("db");
        let u = create_user(&db, "team-del@test.local").await.expect("u");
        let svc = team_service(&db);
        let shared = svc
            .create_shared_team_for_user(
                &u,
                CreateTeam { name: "ToRemove".into(), members: vec![] },
            )
            .await
            .expect("shared");
        let perms = UserPermissions::new(&u, &svc.resolver);
        svc.delete_team_for_user(&perms, &shared.id).await.expect("delete");
    }

    /// Verify that the DB-filtered `fetch_teams_for_user` path returns the same set of teams
    /// as the old fetch-all-then-filter-in-Rust path that it replaces.
    #[tokio::test]
    async fn list_teams_matches_fetch_all_then_filter() {
        let db = test_db().await.expect("db");
        let u = create_user(&db, "team-parity@test.local").await.expect("u");
        let other = create_user(&db, "team-parity-other@test.local").await.expect("other");
        let svc = team_service(&db);

        // Create a shared team that the user belongs to
        let _member_team = svc
            .create_shared_team_for_user(
                &u,
                CreateTeam { name: "MemberTeam".into(), members: vec![] },
            )
            .await
            .expect("member team");

        // Create another shared team that the user does NOT belong to
        let _other_team = svc
            .create_shared_team_for_user(
                &other,
                CreateTeam { name: "OtherTeam".into(), members: vec![] },
            )
            .await
            .expect("other team");

        // New path: DB-filtered
        let new_path_ids: std::collections::BTreeSet<String> = svc
            .list_teams_for_user(&u)
            .await
            .expect("list new")
            .into_iter()
            .map(|t| t.id)
            .collect();

        // Old path: fetch all, filter in Rust
        let app_admin = u.role == shared::user::Role::Admin;
        let old_path_ids: std::collections::BTreeSet<String> = svc
            .repo
            .fetch_all_teams()
            .await
            .expect("fetch all")
            .into_iter()
            .filter(|row| {
                let stored = team_fetched_to_stored(row).expect("stored");
                can_read_team(&u.id, &stored, app_admin)
            })
            .map(|row| row.id.id.to_string())
            .collect();

        assert_eq!(new_path_ids, old_path_ids, "DB-filtered list must match Rust-side filter");
    }
}
