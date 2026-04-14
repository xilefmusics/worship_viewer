use std::collections::BTreeMap;
use std::sync::Arc;

use uuid::Uuid;

use shared::team::{Team, TeamInvitation};
use shared::user::User;

use crate::database::Database;
use crate::error::AppError;

use super::model::{invitation_thing, team_things_match};
use super::repository::TeamInvitationRepository;
use super::surreal_repo::SurrealTeamInvitationRepo;
use crate::resources::team::model::{
    DbTeamMember, is_public_resource, member_or_owner_readable, effective_admin,
    team_fetched_to_stored, team_resource_or_reject_public, thing_user_id, user_thing,
};
use crate::resources::team::repository::TeamRepository;
use crate::resources::team::surreal_repo::SurrealTeamRepo;

/// Application service for team invitation management.
#[derive(Clone)]
pub struct InvitationService<R, IR> {
    pub team_repo: R,
    pub inv_repo: IR,
}

impl<R, IR> InvitationService<R, IR> {
    pub fn new(team_repo: R, inv_repo: IR) -> Self {
        Self { team_repo, inv_repo }
    }
}

impl<R: TeamRepository, IR: TeamInvitationRepository> InvitationService<R, IR> {
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
            return self.team_repo.load_team_display(&team_id_str).await;
        }

        map.insert(
            uid.clone(),
            DbTeamMember { user: user_thing(&uid), role: "guest".to_owned() },
        );
        let members: Vec<DbTeamMember> = map.into_values().collect();
        let resource = (team_row.id.tb.clone(), crate::database::record_id_string(&team_row.id));
        self.team_repo.update_team_members(resource, members).await?;

        self.team_repo.load_team_display(&team_id_str).await
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
            .team_repo
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
pub type InvitationServiceHandle = InvitationService<SurrealTeamRepo, SurrealTeamInvitationRepo>;

impl InvitationServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        InvitationService::new(
            SurrealTeamRepo::new(db.clone()),
            SurrealTeamInvitationRepo::new(db.clone()),
        )
    }
}
