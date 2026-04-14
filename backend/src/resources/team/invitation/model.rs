use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use shared::team::{TeamInvitation, TeamUser};

use crate::database::record_id_string;
use crate::error::AppError;
use crate::resources::user::UserRecord;

#[derive(Debug, Deserialize)]
pub(crate) struct InvitationRow {
    pub(crate) id: Thing,
    pub(crate) team: Thing,
    pub(crate) created_by: UserRecord,
    pub(crate) created_at: Datetime,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvitationAcceptRow {
    #[allow(dead_code)]
    pub(crate) id: Thing,
    pub(crate) team: super::super::model::TeamFetched,
    #[allow(dead_code)]
    pub(crate) created_by: UserRecord,
    #[allow(dead_code)]
    pub(crate) created_at: Datetime,
}

#[derive(Serialize)]
pub(crate) struct InvitationCreate {
    pub(crate) team: Thing,
    pub(crate) created_by: Thing,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvitationIdRow {
    #[allow(dead_code)]
    pub(crate) id: Thing,
}

/// Return shape of `DELETE team_invitation:…` — raw links are not expanded like `SELECT … FETCH`.
#[derive(Debug, Deserialize)]
pub(crate) struct InvitationDeleteRow {
    #[allow(dead_code)]
    pub(crate) id: Thing,
}

impl InvitationRow {
    pub(crate) fn into_invitation(self) -> Result<TeamInvitation, AppError> {
        let u = self.created_by.into_user();
        Ok(TeamInvitation {
            id: record_id_string(&self.id),
            team_id: record_id_string(&self.team),
            created_by: TeamUser {
                id: u.id,
                email: u.email,
            },
            created_at: self.created_at.into(),
        })
    }
}

pub(crate) fn invitation_thing(invitation_id: &str) -> Result<Thing, AppError> {
    let id = invitation_id.trim();
    if id.is_empty() {
        return Err(AppError::NotFound("invitation not found".into()));
    }
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "team_invitation"
    {
        return Ok(thing);
    }
    Ok(Thing::from(("team_invitation".to_owned(), id.to_owned())))
}

pub(crate) fn team_things_match(a: &Thing, b: &Thing) -> bool {
    a.tb == b.tb && record_id_string(a) == record_id_string(b)
}
