use std::collections::BTreeMap;

use std::sync::Arc;
use uuid::Uuid;

use shared::team::{CreateTeam, Team, TeamInvitation, UpdateTeam};
use shared::user::{Role as UserRole, User};

use crate::database::Database;
use crate::error::AppError;

use super::invitation_model::{invitation_thing, team_things_match};
use super::invitation_repository::TeamInvitationRepository;
use super::invitation_surreal_repo::SurrealTeamInvitationRepo;
use super::model::{
    DbTeamMember, TeamCreatePayload, build_create_shared_members, can_read_team, effective_admin,
    ensure_shared_team_has_admin_after_update, inputs_to_db_members, is_public_resource,
    member_or_owner_readable, member_self_leave_payload, team_fetched_to_stored,
    team_resource_or_reject_public, thing_user_id, user_thing, validate_personal_members_not_owner,
};
use super::repository::TeamRepository;
use super::resolver::{TeamResolver, UserPermissions};
use super::surreal_repo::SurrealTeamRepo;

/// Application service: authorization and orchestration for teams and invitations.
#[derive(Clone)]
pub struct TeamService<R, IR, TR> {
    pub repo: R,
    pub inv_repo: IR,
    pub resolver: TR,
}

impl<R, IR, TR> TeamService<R, IR, TR> {
    pub fn new(repo: R, inv_repo: IR, resolver: TR) -> Self {
        Self { repo, inv_repo, resolver }
    }
}

impl<R: TeamRepository, IR: TeamInvitationRepository, TR: TeamResolver> TeamService<R, IR, TR> {
    pub async fn list_teams_for_user(&self, user: &User) -> Result<Vec<Team>, AppError> {
        let app_admin = user.role == UserRole::Admin;
        let rows = self.repo.fetch_all_teams().await?;
        let mut by_id: BTreeMap<String, Team> = BTreeMap::new();
        for row in rows {
            let stored = team_fetched_to_stored(&row)?;
            if can_read_team(&user.id, &stored, app_admin) {
                let team = row.into_team()?;
                by_id.insert(team.id.clone(), team);
            }
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

    // --- Invitation operations ---

    pub async fn create_invitation_for_user(
        &self,
        user: &User,
        team_id: &str,
    ) -> Result<TeamInvitation, AppError> {
        let team_thing = self.assert_shared_team_admin(&user.id, team_id).await?;
        let inv_id = Uuid::new_v4().to_string();
        self.inv_repo
            .create_invitation(team_thing, user_thing(&user.id), &inv_id)
            .await?;
        self.get_invitation_for_user(user, team_id, &inv_id).await
    }

    pub async fn list_invitations_for_user(
        &self,
        user: &User,
        team_id: &str,
    ) -> Result<Vec<TeamInvitation>, AppError> {
        let team_thing = self.assert_shared_team_admin(&user.id, team_id).await?;
        let rows = self.inv_repo.list_invitations(team_thing).await?;
        rows.into_iter()
            .map(|r| r.into_invitation())
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_invitation_for_user(
        &self,
        user: &User,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<TeamInvitation, AppError> {
        let team_thing = self.assert_shared_team_admin(&user.id, team_id).await?;
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .inv_repo
            .get_invitation(&inv_thing.id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        if !team_things_match(&row.team, &team_thing) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        row.into_invitation()
    }

    pub async fn delete_invitation_for_user(
        &self,
        user: &User,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<(), AppError> {
        let team_thing = self.assert_shared_team_admin(&user.id, team_id).await?;
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .inv_repo
            .get_invitation(&inv_thing.id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        if !team_things_match(&row.team, &team_thing) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let deleted = self
            .inv_repo
            .delete_invitation(&inv_thing.id.to_string())
            .await?;
        if !deleted {
            return Err(AppError::NotFound("invitation not found".into()));
        }
        Ok(())
    }

    pub async fn accept_invitation_for_user(
        &self,
        user: &User,
        invitation_id: &str,
    ) -> Result<Team, AppError> {
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .inv_repo
            .get_invitation_with_team(&inv_thing.id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        let team_row = row.team;
        let res = (team_row.id.tb.clone(), crate::database::record_id_string(&team_row.id));
        if is_public_resource(&res) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let stored = team_fetched_to_stored(&team_row)?;
        if stored.owner.is_some() {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let team_id_str = crate::database::record_id_string(&team_row.id);
        let uid = user.id.clone();
        let mut map: BTreeMap<String, DbTeamMember> = BTreeMap::new();
        for m in &stored.members {
            map.insert(thing_user_id(&m.user), m.clone());
        }
        let needs_guest = match map.get(&uid).map(|m| m.role.as_str()) {
            Some("admin") | Some("content_maintainer") | Some("guest") => false,
            None => true,
            Some(_) => true,
        };

        if !needs_guest {
            return self.repo.load_team_display(&team_id_str).await;
        }

        map.insert(
            uid.clone(),
            DbTeamMember { user: user_thing(&uid), role: "guest".to_owned() },
        );
        let members: Vec<DbTeamMember> = map.into_values().collect();
        let resource = (team_row.id.tb.clone(), crate::database::record_id_string(&team_row.id));
        self.repo.update_team_members(resource, members).await?;

        self.repo.load_team_display(&team_id_str).await
    }

    /// Asserts that a team exists, is a shared team, and the user is an admin of it.
    /// Returns the team `Thing` for binding into queries.
    async fn assert_shared_team_admin(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<surrealdb::sql::Thing, AppError> {
        let resource = team_resource_or_reject_public(team_id)?;
        let team_thing = surrealdb::sql::Thing::from(resource);
        let row = self
            .repo
            .fetch_team(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("team not found".into()))?;

        let stored = team_fetched_to_stored(&row)?;
        if stored.owner.is_some() {
            return Err(AppError::invalid_request(
                "team invitations are only supported for shared teams",
            ));
        }
        if !member_or_owner_readable(user_id, &stored) {
            return Err(AppError::NotFound("team not found".into()));
        }
        if !effective_admin(user_id, &stored) {
            return Err(AppError::forbidden());
        }
        Ok(team_thing)
    }
}

/// Production type alias used in HTTP wiring.
pub type TeamServiceHandle = TeamService<
    SurrealTeamRepo,
    SurrealTeamInvitationRepo,
    super::resolver::SurrealTeamResolver,
>;

impl TeamServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        TeamService::new(
            SurrealTeamRepo::new(db.clone()),
            SurrealTeamInvitationRepo::new(db.clone()),
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
}
