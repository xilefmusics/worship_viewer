use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::sql::Thing;

use crate::database::Database;
use crate::error::AppError;

use super::model::{
    InvitationAcceptRow, InvitationCreate, InvitationDeleteRow, InvitationIdRow, InvitationRow,
};
use super::repository::TeamInvitationRepository;

#[derive(Clone)]
pub struct SurrealTeamInvitationRepo {
    db: Arc<Database>,
}

impl SurrealTeamInvitationRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl TeamInvitationRepository for SurrealTeamInvitationRepo {
    async fn create_invitation(
        &self,
        team: Thing,
        created_by: Thing,
        inv_id: &str,
    ) -> Result<(), AppError> {
        let create = InvitationCreate { team, created_by };
        let created: Option<InvitationIdRow> = self
            .inner()
            .db
            .create(("team_invitation", inv_id))
            .content(create)
            .await
            .map_err(AppError::database)?;
        if created.is_none() {
            return Err(AppError::database("failed to create team invitation"));
        }
        Ok(())
    }

    async fn list_invitations(&self, team: Thing) -> Result<Vec<InvitationRow>, AppError> {
        Ok(self
            .inner()
            .db
            .query(
                "SELECT * FROM team_invitation WHERE team = $team ORDER BY created_at ASC FETCH created_by",
            )
            .bind(("team", team))
            .await?
            .take(0)?)
    }

    async fn get_invitation(&self, inv_id: &str) -> Result<Option<InvitationRow>, AppError> {
        let inv_thing = invitation_thing_from_id(inv_id)?;
        Ok(self
            .inner()
            .db
            .query("SELECT * FROM $iid FETCH created_by")
            .bind(("iid", inv_thing))
            .await?
            .take::<Option<InvitationRow>>(0)?)
    }

    async fn delete_invitation(&self, inv_id: &str) -> Result<bool, AppError> {
        let inv_thing = invitation_thing_from_id(inv_id)?;
        let key = crate::database::record_id_string(&inv_thing);
        let deleted: Option<InvitationDeleteRow> = self
            .inner()
            .db
            .delete(("team_invitation", key.as_str()))
            .await?;
        Ok(deleted.is_some())
    }

    async fn get_invitation_with_team(
        &self,
        inv_id: &str,
    ) -> Result<Option<InvitationAcceptRow>, AppError> {
        let inv_thing = invitation_thing_from_id(inv_id)?;
        // `FETCH team` alone leaves `members[].user` as bare ids; use `team.members.user` to fully expand.
        Ok(self
            .inner()
            .db
            .query("SELECT * FROM $iid FETCH created_by, team, team.members.user")
            .bind(("iid", inv_thing))
            .await?
            .take::<Option<InvitationAcceptRow>>(0)?)
    }
}

fn invitation_thing_from_id(id: &str) -> Result<Thing, crate::error::AppError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(crate::error::AppError::NotFound(
            "invitation not found".into(),
        ));
    }
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "team_invitation"
    {
        return Ok(thing);
    }
    Ok(Thing::from(("team_invitation".to_owned(), id.to_owned())))
}
