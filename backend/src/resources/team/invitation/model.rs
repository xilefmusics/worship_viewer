use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use shared::team::{TeamInvitation, TeamUser};

use crate::database::record_id_string;
use crate::error::AppError;
use crate::resources::user::UserRecord;

#[derive(Clone, Debug, Deserialize, SurrealValue)]
pub struct InvitationRow {
    pub id: RecordId,
    pub team: RecordId,
    pub created_by: UserRecord,
    pub created_at: Datetime,
}

#[derive(Clone, Debug, Deserialize, SurrealValue)]
pub struct InvitationAcceptRow {
    pub team: super::super::model::TeamFetched,
}

#[derive(Serialize, SurrealValue)]
pub struct InvitationCreate {
    pub team: RecordId,
    pub created_by: RecordId,
}

impl InvitationRow {
    pub fn into_invitation(self) -> Result<TeamInvitation, AppError> {
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

pub fn invitation_thing(invitation_id: &str) -> Result<RecordId, AppError> {
    let id = invitation_id.trim();
    if id.is_empty() {
        return Err(AppError::NotFound("invitation not found".into()));
    }
    if let Ok(rid) = RecordId::parse_simple(id)
        && rid.table.as_str() == "team_invitation"
    {
        return Ok(rid);
    }
    Ok(RecordId::new("team_invitation", id))
}

pub fn team_things_match(a: &RecordId, b: &RecordId) -> bool {
    a.table == b.table && record_id_string(a) == record_id_string(b)
}

#[cfg(test)]
mod tests {
    use surrealdb::types::RecordId;

    use super::*;
    use crate::error::AppError;

    fn make_thing(table: &str, id: &str) -> RecordId {
        RecordId::new(table, id.to_owned())
    }

    /// BLC-TINV-006: plain UUID produces a team_invitation RecordId.
    #[test]
    fn blc_tinv_006_invitation_thing_plain_uuid_ok() {
        let thing = invitation_thing("valid-uuid").unwrap();
        assert_eq!(thing.table.as_str(), "team_invitation");
        assert_eq!(record_id_string(&thing), "valid-uuid");
    }

    /// Already-prefixed "team_invitation:abc" is parsed without wrapping.
    #[test]
    fn invitation_thing_prefixed_id_parsed_ok() {
        let thing = invitation_thing("team_invitation:abc").unwrap();
        assert_eq!(thing.table.as_str(), "team_invitation");
        assert_eq!(record_id_string(&thing), "abc");
    }

    /// Empty string returns NotFound.
    #[test]
    fn invitation_thing_empty_string_not_found() {
        let err = invitation_thing("").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    /// Whitespace-only string returns NotFound.
    #[test]
    fn invitation_thing_whitespace_only_not_found() {
        let err = invitation_thing("   ").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    /// A "other_table:abc" string (wrong table prefix) falls back to team_invitation.
    #[test]
    fn invitation_thing_wrong_table_prefix_falls_back() {
        let thing = invitation_thing("other_table:abc").unwrap();
        assert_eq!(thing.table.as_str(), "team_invitation");
    }

    /// Same table and same id returns true.
    #[test]
    fn team_things_match_same_table_same_id_true() {
        let a = make_thing("team", "t1");
        let b = make_thing("team", "t1");
        assert!(team_things_match(&a, &b));
    }

    /// Same table but different id returns false.
    #[test]
    fn team_things_match_same_table_different_id_false() {
        let a = make_thing("team", "t1");
        let b = make_thing("team", "t2");
        assert!(!team_things_match(&a, &b));
    }

    /// Different table returns false.
    #[test]
    fn team_things_match_different_table_false() {
        let a = make_thing("team", "t1");
        let b = make_thing("other_team", "t1");
        assert!(!team_things_match(&a, &b));
    }
}
