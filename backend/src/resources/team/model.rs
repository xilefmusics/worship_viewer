use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb::sql::Thing;

use shared::team::{Team, TeamMember, TeamMemberInput, TeamRole, TeamUser, TeamUserRef};

use crate::database::record_id_string;
use crate::error::AppError;
use crate::resources::user::UserRecord;

pub(crate) fn thing_record_key(t: &Thing) -> String {
    format!("{}:{}", t.tb, record_id_string(t))
}

// Used by `resolver` parity tests; write ACL is enforced in SurrealQL in `content_write_team_things`.
#[allow(dead_code)]
pub(crate) fn team_content_writable(user_id: &str, stored: &TeamStored) -> bool {
    if let Some(ref o) = stored.owner
        && thing_user_id(o) == user_id
    {
        return true;
    }
    stored.members.iter().any(|m| {
        thing_user_id(&m.user) == user_id && (m.role == "admin" || m.role == "content_maintainer")
    })
}

pub(crate) fn build_create_shared_members(
    creator_id: &str,
    extra: &[TeamMemberInput],
) -> Result<Vec<DbTeamMember>, AppError> {
    let mut map: BTreeMap<String, DbTeamMember> = BTreeMap::new();
    map.insert(
        creator_id.to_owned(),
        DbTeamMember {
            user: user_thing(creator_id),
            role: role_str(&TeamRole::Admin).to_owned(),
        },
    );
    for m in extra {
        let uid = member_user_id(&m.user)?;
        if uid == creator_id {
            continue;
        }
        map.insert(
            uid.clone(),
            DbTeamMember {
                user: user_thing(&uid),
                role: role_str(&m.role).to_owned(),
            },
        );
    }
    let members: Vec<DbTeamMember> = map.into_values().collect();
    validate_shared_has_admin(&members)?;
    Ok(members)
}

pub(crate) fn inputs_to_db_members(inputs: &[TeamMemberInput]) -> Result<Vec<DbTeamMember>, AppError> {
    let mut map: BTreeMap<String, DbTeamMember> = BTreeMap::new();
    for m in inputs {
        let uid = member_user_id(&m.user)?;
        map.insert(
            uid.clone(),
            DbTeamMember {
                user: user_thing(&uid),
                role: role_str(&m.role).to_owned(),
            },
        );
    }
    Ok(map.into_values().collect())
}

pub(crate) fn member_user_id(user: &TeamUserRef) -> Result<String, AppError> {
    let id = user.id.trim();
    if id.is_empty() {
        return Err(AppError::invalid_request(
            "member user id must not be empty",
        ));
    }
    Ok(id.to_owned())
}

pub(crate) fn validate_shared_has_admin(members: &[DbTeamMember]) -> Result<(), AppError> {
    if !members.iter().any(|m| m.role == "admin") {
        return Err(AppError::invalid_request(
            "shared team must have at least one admin member",
        ));
    }
    Ok(())
}

/// After a membership update on an existing shared team (PUT), missing any admin is a conflict (e.g. sole admin leaving).
pub(crate) fn ensure_shared_team_has_admin_after_update(members: &[DbTeamMember]) -> Result<(), AppError> {
    if !members.iter().any(|m| m.role == "admin") {
        return Err(AppError::conflict(
            "cannot leave team as the sole admin; promote another admin or delete the team",
        ));
    }
    Ok(())
}

fn members_role_map(members: &[DbTeamMember]) -> BTreeMap<String, String> {
    members
        .iter()
        .map(|x| (thing_user_id(&x.user), x.role.clone()))
        .collect()
}

fn members_without_user(stored: &TeamStored, user_id: &str) -> Vec<DbTeamMember> {
    let u = user_thing(user_id);
    stored
        .members
        .iter()
        .filter(|m| m.user != u)
        .cloned()
        .collect()
}

/// Non-admins may only PUT to remove themselves: same team name and `members` exactly the current list minus the caller.
pub(crate) fn member_self_leave_payload(
    stored: &TeamStored,
    user_id: &str,
    current_name: &str,
    payload_name: &str,
    new_members: &[DbTeamMember],
) -> bool {
    let u = user_thing(user_id);
    if !stored.members.iter().any(|m| m.user == u) {
        return false;
    }
    if current_name.trim() != payload_name.trim() {
        return false;
    }
    let expected = members_without_user(stored, user_id);
    members_role_map(new_members) == members_role_map(&expected)
}

pub(crate) fn validate_personal_members_not_owner(
    owner_id: &str,
    members: &[DbTeamMember],
) -> Result<(), AppError> {
    let o_thing = user_thing(owner_id);
    if members.iter().any(|m| m.user == o_thing) {
        return Err(AppError::invalid_request(
            "personal team owner must not appear in members",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
pub(crate) struct TeamCreatePayload {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) owner: Option<Thing>,
    pub(crate) members: Vec<DbTeamMember>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DbTeamMember {
    pub(crate) user: Thing,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TeamFetched {
    pub(crate) id: Thing,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) owner: Option<UserRecord>,
    #[serde(default)]
    pub(crate) members: Vec<TeamMemberFetched>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TeamMemberFetched {
    pub(crate) user: UserRecord,
    pub(crate) role: String,
}

impl TeamFetched {
    pub(crate) fn into_team(self) -> Result<Team, AppError> {
        let id = self.id.id.to_string();
        let owner = self.owner.map(user_record_to_team_user).transpose()?;
        let mut members = Vec::with_capacity(self.members.len());
        for m in self.members {
            members.push(TeamMember {
                user: user_record_to_team_user(m.user)?,
                role: parse_role(&m.role)?,
            });
        }
        Ok(Team {
            id,
            owner,
            name: self.name,
            members,
        })
    }
}

fn user_record_to_team_user(rec: UserRecord) -> Result<TeamUser, AppError> {
    let u = rec.into_user();
    Ok(TeamUser {
        id: u.id,
        email: u.email,
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TeamIdRow {
    pub(crate) id: Thing,
}

#[derive(Clone, Debug)]
pub(crate) struct TeamStored {
    pub(crate) owner: Option<Thing>,
    pub(crate) members: Vec<DbTeamMember>,
}

pub(crate) fn team_fetched_to_stored(row: &TeamFetched) -> Result<TeamStored, AppError> {
    let owner = row
        .owner
        .as_ref()
        .map(|u| user_thing(&u.clone().into_user().id));
    let mut members = Vec::new();
    for m in &row.members {
        let uid = m.user.clone().into_user().id;
        members.push(DbTeamMember {
            user: user_thing(&uid),
            role: m.role.clone(),
        });
    }
    Ok(TeamStored { owner, members })
}

pub(crate) fn user_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

pub(crate) fn public_team_thing() -> Thing {
    Thing::from(("team".to_owned(), "public".to_owned()))
}

pub(crate) fn is_public_resource(resource: &(String, String)) -> bool {
    resource.0 == "team" && resource.1 == "public"
}

/// `team:public` is seeded for internal use only (see migration). It is not exposed through the REST API.
pub(crate) fn team_resource_or_reject_public(id: &str) -> Result<(String, String), AppError> {
    let resource = team_resource(id)?;
    if is_public_resource(&resource) {
        return Err(AppError::NotFound("team not found".into()));
    }
    Ok(resource)
}

fn team_resource(id: &str) -> Result<(String, String), AppError> {
    if id == "public" {
        return Ok(("team".to_owned(), "public".to_owned()));
    }
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "team"
    {
        return Ok((thing.tb, thing.id.to_string()));
    }
    Ok(("team".to_owned(), id.to_owned()))
}

pub(crate) fn thing_user_id(t: &Thing) -> String {
    t.id.to_string()
}

pub(crate) fn member_or_owner_readable(user_id: &str, stored: &TeamStored) -> bool {
    if let Some(ref o) = stored.owner
        && thing_user_id(o) == user_id
    {
        return true;
    }
    stored
        .members
        .iter()
        .any(|m| thing_user_id(&m.user) == user_id)
}

/// List/get team: members, personal owner, or platform (`User.role` admin) for read-only API access.
pub(crate) fn can_read_team(user_id: &str, stored: &TeamStored, app_admin: bool) -> bool {
    app_admin || member_or_owner_readable(user_id, stored)
}

pub(crate) fn effective_admin(user_id: &str, stored: &TeamStored) -> bool {
    if let Some(ref o) = stored.owner
        && thing_user_id(o) == user_id
    {
        return true;
    }
    stored
        .members
        .iter()
        .any(|m| m.role == "admin" && thing_user_id(&m.user) == user_id)
}

fn parse_role(s: &str) -> Result<TeamRole, AppError> {
    match s {
        "guest" => Ok(TeamRole::Guest),
        "content_maintainer" => Ok(TeamRole::ContentMaintainer),
        "admin" => Ok(TeamRole::Admin),
        _ => Err(AppError::invalid_request("invalid team role")),
    }
}

fn role_str(r: &TeamRole) -> &'static str {
    match r {
        TeamRole::Guest => "guest",
        TeamRole::ContentMaintainer => "content_maintainer",
        TeamRole::Admin => "admin",
    }
}

