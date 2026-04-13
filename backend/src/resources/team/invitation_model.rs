use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};
use uuid::Uuid;

use shared::team::{Team, TeamInvitation, TeamUser};

use crate::database::{Database, record_id_string};
use crate::error::AppError;
use crate::resources::user::UserRecord;

use super::model::{
    DbTeamMember, TeamFetched, effective_admin, is_public_resource, load_team_display,
    member_or_owner_readable, team_fetched_to_stored, team_resource_or_reject_public,
    thing_user_id, user_thing,
};

pub trait TeamInvitationModel {
    async fn create_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<TeamInvitation, AppError>;

    async fn list_team_invitations(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<Vec<TeamInvitation>, AppError>;

    async fn get_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<TeamInvitation, AppError>;

    async fn delete_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<(), AppError>;

    async fn accept_team_invitation(
        &self,
        user_id: &str,
        invitation_id: &str,
    ) -> Result<Team, AppError>;
}

#[derive(Debug, Deserialize)]
struct InvitationRow {
    id: Thing,
    team: Thing,
    created_by: UserRecord,
    created_at: Datetime,
}

#[derive(Debug, Deserialize)]
struct InvitationAcceptRow {
    #[allow(dead_code)]
    id: Thing,
    team: TeamFetched,
    #[allow(dead_code)]
    created_by: UserRecord,
    #[allow(dead_code)]
    created_at: Datetime,
}

#[derive(Serialize)]
struct InvitationCreate {
    team: Thing,
    created_by: Thing,
}

#[derive(Debug, Deserialize)]
struct InvitationIdRow {
    #[allow(dead_code)]
    id: Thing,
}

impl InvitationRow {
    fn into_invitation(self) -> Result<TeamInvitation, AppError> {
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

async fn assert_shared_team_admin(
    db: &Database,
    user_id: &str,
    team_id: &str,
) -> Result<Thing, AppError> {
    let resource = team_resource_or_reject_public(team_id)?;
    let team_thing = Thing::from(resource.clone());
    let row = db
        .db
        .query("SELECT * FROM $tid FETCH owner, members.user")
        .bind(("tid", team_thing.clone()))
        .await?
        .take::<Option<TeamFetched>>(0)?
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

fn invitation_thing(invitation_id: &str) -> Result<Thing, AppError> {
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

fn team_things_match(a: &Thing, b: &Thing) -> bool {
    a.tb == b.tb && record_id_string(a) == record_id_string(b)
}

impl TeamInvitationModel for Database {
    async fn create_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<TeamInvitation, AppError> {
        let team_thing = assert_shared_team_admin(self, user_id, team_id).await?;
        let inv_id = Uuid::new_v4().to_string();
        let create = InvitationCreate {
            team: team_thing.clone(),
            created_by: user_thing(user_id),
        };
        let created: Option<InvitationIdRow> = self
            .db
            .create(("team_invitation", inv_id.as_str()))
            .content(create)
            .await
            .map_err(AppError::database)?;
        if created.is_none() {
            return Err(AppError::database("failed to create team invitation"));
        }

        self.get_team_invitation(user_id, team_id, &inv_id).await
    }

    async fn list_team_invitations(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<Vec<TeamInvitation>, AppError> {
        let team_thing = assert_shared_team_admin(self, user_id, team_id).await?;
        let rows: Vec<InvitationRow> = self
            .db
            .query(
                "SELECT * FROM team_invitation WHERE team = $team ORDER BY created_at ASC FETCH created_by",
            )
            .bind(("team", team_thing))
            .await?
            .take(0)?;
        rows.into_iter()
            .map(InvitationRow::into_invitation)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<TeamInvitation, AppError> {
        let team_thing = assert_shared_team_admin(self, user_id, team_id).await?;
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .db
            .query("SELECT * FROM $iid FETCH created_by")
            .bind(("iid", inv_thing))
            .await?
            .take::<Option<InvitationRow>>(0)?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        if !team_things_match(&row.team, &team_thing) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        row.into_invitation()
    }

    async fn delete_team_invitation(
        &self,
        user_id: &str,
        team_id: &str,
        invitation_id: &str,
    ) -> Result<(), AppError> {
        let team_thing = assert_shared_team_admin(self, user_id, team_id).await?;
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .db
            .query("SELECT * FROM $iid FETCH created_by")
            .bind(("iid", inv_thing.clone()))
            .await?
            .take::<Option<InvitationRow>>(0)?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        if !team_things_match(&row.team, &team_thing) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let key = record_id_string(&inv_thing);
        let deleted: Option<InvitationRow> =
            self.db.delete(("team_invitation", key.as_str())).await?;
        if deleted.is_none() {
            return Err(AppError::NotFound("invitation not found".into()));
        }
        Ok(())
    }

    async fn accept_team_invitation(
        &self,
        user_id: &str,
        invitation_id: &str,
    ) -> Result<Team, AppError> {
        let inv_thing = invitation_thing(invitation_id)?;
        let row = self
            .db
            .query("SELECT * FROM $iid FETCH team, created_by")
            .bind(("iid", inv_thing))
            .await?
            .take::<Option<InvitationAcceptRow>>(0)?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        let team_row = row.team;
        let res = (team_row.id.tb.clone(), record_id_string(&team_row.id));
        if is_public_resource(&res) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let stored = team_fetched_to_stored(&team_row)?;
        if stored.owner.is_some() {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let team_id_str = record_id_string(&team_row.id);
        let uid = user_id.to_owned();
        let mut map: std::collections::BTreeMap<String, DbTeamMember> =
            std::collections::BTreeMap::new();
        for m in &stored.members {
            map.insert(thing_user_id(&m.user), m.clone());
        }
        let needs_guest = match map.get(&uid).map(|m| m.role.as_str()) {
            Some("admin") | Some("content_maintainer") | Some("guest") => false,
            None => true,
            Some(_) => true,
        };

        if !needs_guest {
            return load_team_display(self, &team_id_str).await;
        }

        map.insert(
            uid.clone(),
            DbTeamMember {
                user: user_thing(&uid),
                role: "guest".to_owned(),
            },
        );
        let members: Vec<DbTeamMember> = map.into_values().collect();

        let tid = team_row.id.clone();
        self.db
            .query("UPDATE $tid SET members = $members")
            .bind(("tid", tid))
            .bind(("members", members))
            .await?
            .check()
            .map_err(AppError::database)?;

        load_team_display(self, &team_id_str).await
    }
}
