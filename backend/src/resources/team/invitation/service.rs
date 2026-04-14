use std::collections::BTreeMap;
use std::sync::Arc;

use uuid::Uuid;

use shared::team::{Team, TeamInvitation};
use shared::user::User;

use crate::database::{record_id_string, Database};
use crate::error::AppError;

use super::model::{invitation_thing, team_things_match};
use super::repository::TeamInvitationRepository;
use super::surreal_repo::SurrealTeamInvitationRepo;
use crate::resources::team::model::{
    DbTeamMember, effective_admin, is_public_resource, member_or_owner_readable,
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
        Self {
            team_repo,
            inv_repo,
        }
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
        let inv_id_key = record_id_string(&inv_thing);
        let row = self
            .inv_repo
            .get_invitation(&inv_id_key)
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
        let inv_id_key = record_id_string(&inv_thing);
        let row = self
            .inv_repo
            .get_invitation(&inv_id_key)
            .await?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        if !team_things_match(&row.team, &team_thing) {
            return Err(AppError::NotFound("invitation not found".into()));
        }

        let deleted = self
            .inv_repo
            .delete_invitation(&inv_id_key)
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
            .get_invitation_with_team(&record_id_string(&inv_thing))
            .await?
            .ok_or_else(|| AppError::NotFound("invitation not found".into()))?;

        let team_row = row.team;
        let res = (
            team_row.id.tb.clone(),
            crate::database::record_id_string(&team_row.id),
        );
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
            DbTeamMember {
                user: user_thing(&uid),
                role: "guest".to_owned(),
            },
        );
        let members: Vec<DbTeamMember> = map.into_values().collect();
        let resource = (
            team_row.id.tb.clone(),
            crate::database::record_id_string(&team_row.id),
        );
        self.team_repo
            .update_team_members(resource, members)
            .await?;

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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use surrealdb::sql::{Datetime, Thing};

    use shared::team::Team;
    use shared::user::User;

    use crate::error::AppError;
    use crate::resources::team::model::{
        DbTeamMember, TeamCreatePayload, TeamFetched, TeamMemberFetched,
    };
    use crate::resources::team::repository::TeamRepository;
    use crate::resources::user::UserRecord;

    use super::InvitationService;
    use super::super::model::{InvitationAcceptRow, InvitationRow};
    use super::super::repository::TeamInvitationRepository;

    // ── Test data helpers ─────────────────────────────────────────────────────

    fn make_user(id: &str) -> User {
        let mut u = User::new("test@example.com");
        u.id = id.to_owned();
        u
    }

    fn team_thing(id: &str) -> Thing {
        Thing::from(("team".to_owned(), id.to_owned()))
    }

    fn inv_thing(id: &str) -> Thing {
        Thing::from(("team_invitation".to_owned(), id.to_owned()))
    }

    fn member_fetched(user_id: &str, role: &str) -> TeamMemberFetched {
        TeamMemberFetched {
            user: UserRecord::from_user(make_user(user_id)),
            role: role.to_owned(),
        }
    }

    fn shared_team(team_id: &str, members: Vec<TeamMemberFetched>) -> TeamFetched {
        TeamFetched {
            id: team_thing(team_id),
            name: "Shared Team".to_owned(),
            owner: None,
            members,
        }
    }

    fn personal_team(team_id: &str, owner_id: &str) -> TeamFetched {
        TeamFetched {
            id: team_thing(team_id),
            name: "Personal".to_owned(),
            owner: Some(UserRecord::from_user(make_user(owner_id))),
            members: vec![],
        }
    }

    fn public_team_fetched() -> TeamFetched {
        TeamFetched {
            id: Thing::from(("team".to_owned(), "public".to_owned())),
            name: "Public".to_owned(),
            owner: None,
            members: vec![],
        }
    }

    fn team_display() -> Team {
        Team {
            id: "t1".to_owned(),
            owner: None,
            name: "Shared Team".to_owned(),
            members: vec![],
        }
    }

    fn inv_row(inv_id: &str, for_team_id: &str) -> InvitationRow {
        InvitationRow {
            id: inv_thing(inv_id),
            team: team_thing(for_team_id),
            created_by: UserRecord::from_user(make_user("creator")),
            created_at: Datetime::default(),
        }
    }

    fn inv_accept_row(inv_id: &str, team: TeamFetched) -> InvitationAcceptRow {
        InvitationAcceptRow {
            id: inv_thing(inv_id),
            team,
            created_by: UserRecord::from_user(make_user("creator")),
            created_at: Datetime::default(),
        }
    }

    // ── MockTeamRepo ──────────────────────────────────────────────────────────

    struct MockTeamRepo {
        team: Option<TeamFetched>,
        display: Team,
        update_members_called: Arc<Mutex<bool>>,
    }

    impl MockTeamRepo {
        fn with(team: TeamFetched) -> Self {
            Self {
                team: Some(team),
                display: team_display(),
                update_members_called: Arc::new(Mutex::new(false)),
            }
        }

        fn missing() -> Self {
            Self {
                team: None,
                display: team_display(),
                update_members_called: Arc::new(Mutex::new(false)),
            }
        }
    }

    #[async_trait]
    impl TeamRepository for MockTeamRepo {
        async fn fetch_team(&self, _id: &str) -> Result<Option<TeamFetched>, AppError> {
            Ok(self.team.clone())
        }

        async fn load_team_display(&self, _id: &str) -> Result<Team, AppError> {
            Ok(self.display.clone())
        }

        async fn update_team_members(
            &self,
            _resource: (String, String),
            _members: Vec<DbTeamMember>,
        ) -> Result<(), AppError> {
            *self.update_members_called.lock().unwrap() = true;
            Ok(())
        }

        async fn create_team(&self, _payload: TeamCreatePayload) -> Result<String, AppError> {
            unreachable!("not used in invitation tests")
        }

        async fn fetch_all_teams(&self) -> Result<Vec<TeamFetched>, AppError> {
            unreachable!("not used in invitation tests")
        }

        async fn fetch_teams_for_user(
            &self,
            _user_id: &str,
            _is_admin: bool,
        ) -> Result<Vec<TeamFetched>, AppError> {
            unreachable!("not used in invitation tests")
        }

        async fn update_team_name(
            &self,
            _resource: (String, String),
            _name: &str,
        ) -> Result<(), AppError> {
            unreachable!("not used in invitation tests")
        }

        async fn delete_team_record(
            &self,
            _resource: (String, String),
        ) -> Result<(), AppError> {
            unreachable!("not used in invitation tests")
        }

        async fn reassign_content(&self, _from: Thing, _to: Thing) -> Result<(), AppError> {
            unreachable!("not used in invitation tests")
        }
    }

    // ── MockInvRepo ───────────────────────────────────────────────────────────

    struct MockInvRepo {
        invitation: Option<InvitationRow>,
        inv_with_team: Option<InvitationAcceptRow>,
        delete_ok: bool,
        list: Vec<InvitationRow>,
    }

    impl MockInvRepo {
        fn empty() -> Self {
            Self {
                invitation: None,
                inv_with_team: None,
                delete_ok: false,
                list: vec![],
            }
        }

        fn with_inv(row: InvitationRow) -> Self {
            Self {
                invitation: Some(row),
                inv_with_team: None,
                delete_ok: true,
                list: vec![],
            }
        }

        fn with_accept(row: InvitationAcceptRow) -> Self {
            Self {
                invitation: None,
                inv_with_team: Some(row),
                delete_ok: false,
                list: vec![],
            }
        }

        fn with_list(rows: Vec<InvitationRow>) -> Self {
            Self {
                invitation: None,
                inv_with_team: None,
                delete_ok: false,
                list: rows,
            }
        }
    }

    #[async_trait]
    impl TeamInvitationRepository for MockInvRepo {
        async fn create_invitation(
            &self,
            _team: Thing,
            _created_by: Thing,
            _inv_id: &str,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn list_invitations(&self, _team: Thing) -> Result<Vec<InvitationRow>, AppError> {
            Ok(self.list.clone())
        }

        async fn get_invitation(&self, _inv_id: &str) -> Result<Option<InvitationRow>, AppError> {
            Ok(self.invitation.clone())
        }

        async fn delete_invitation(&self, _inv_id: &str) -> Result<bool, AppError> {
            Ok(self.delete_ok)
        }

        async fn get_invitation_with_team(
            &self,
            _inv_id: &str,
        ) -> Result<Option<InvitationAcceptRow>, AppError> {
            Ok(self.inv_with_team.clone())
        }
    }

    fn make_svc(
        team: MockTeamRepo,
        inv: MockInvRepo,
    ) -> InvitationService<MockTeamRepo, MockInvRepo> {
        InvitationService::new(team, inv)
    }

    // ── Slice 2B: CRUD access control ─────────────────────────────────────────

    /// BLC-TINV-001, BLC-TINV-007: creating an invitation for a shared team as admin succeeds.
    #[tokio::test]
    async fn blc_tinv_001_create_shared_team_admin_ok() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(
            MockTeamRepo::with(team),
            MockInvRepo::with_inv(inv_row("any", "t1")),
        );
        let r = svc.create_invitation_for_user(&user, "t1").await;
        assert!(r.is_ok());
    }

    /// BLC-TINV-001: creating an invitation for a personal team is rejected with invalid request.
    #[tokio::test]
    async fn blc_tinv_001_create_personal_team_rejected() {
        let user = make_user("u1");
        let team = personal_team("t1", "u1");
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.create_invitation_for_user(&user, "t1").await;
        assert!(matches!(r, Err(AppError::InvalidRequest(_))));
    }

    /// BLC-TINV-001: creating an invitation for team:public is rejected before DB fetch.
    #[tokio::test]
    async fn blc_tinv_001_create_public_team_rejected() {
        let user = make_user("u1");
        let svc = make_svc(MockTeamRepo::missing(), MockInvRepo::empty());
        let r = svc.create_invitation_for_user(&user, "public").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-002: content_maintainer cannot create an invitation.
    #[tokio::test]
    async fn blc_tinv_002_create_content_maintainer_forbidden() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "content_maintainer")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.create_invitation_for_user(&user, "t1").await;
        assert!(matches!(r, Err(AppError::Forbidden)));
    }

    /// BLC-TINV-002: guest cannot create an invitation.
    #[tokio::test]
    async fn blc_tinv_002_create_guest_forbidden() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "guest")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.create_invitation_for_user(&user, "t1").await;
        assert!(matches!(r, Err(AppError::Forbidden)));
    }

    /// BLC-TINV-002: non-member gets not found.
    #[tokio::test]
    async fn blc_tinv_002_create_non_member_not_found() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u2", "admin")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.create_invitation_for_user(&user, "t1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-002, BLC-TINV-008: admin can list invitations for their team.
    #[tokio::test]
    async fn blc_tinv_002_list_admin_ok() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(
            MockTeamRepo::with(team),
            MockInvRepo::with_list(vec![inv_row("inv1", "t1")]),
        );
        let r = svc.list_invitations_for_user(&user, "t1").await;
        assert!(r.is_ok());
        assert_eq!(r.unwrap().len(), 1);
    }

    /// BLC-TINV-002: non-admin (guest) cannot list invitations.
    #[tokio::test]
    async fn blc_tinv_002_list_non_admin_forbidden() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "guest")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.list_invitations_for_user(&user, "t1").await;
        assert!(matches!(r, Err(AppError::Forbidden)));
    }

    /// BLC-TINV-008: admin can get an invitation by id when team matches.
    #[tokio::test]
    async fn blc_tinv_008_get_admin_correct_team_ok() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(
            MockTeamRepo::with(team),
            MockInvRepo::with_inv(inv_row("inv1", "t1")),
        );
        let r = svc.get_invitation_for_user(&user, "t1", "inv1").await;
        assert!(r.is_ok());
    }

    /// BLC-TINV-008: invitation belonging to a different team returns not found.
    #[tokio::test]
    async fn blc_tinv_008_get_admin_wrong_team_not_found() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(
            MockTeamRepo::with(team),
            MockInvRepo::with_inv(inv_row("inv1", "t2")),
        );
        let r = svc.get_invitation_for_user(&user, "t1", "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-008: non-existent invitation id returns not found.
    #[tokio::test]
    async fn blc_tinv_008_get_nonexistent_not_found() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.get_invitation_for_user(&user, "t1", "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-009: admin can delete an invitation.
    #[tokio::test]
    async fn blc_tinv_009_delete_admin_ok() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(
            MockTeamRepo::with(team),
            MockInvRepo::with_inv(inv_row("inv1", "t1")),
        );
        let r = svc.delete_invitation_for_user(&user, "t1", "inv1").await;
        assert!(r.is_ok());
    }

    /// BLC-TINV-009: deleting a non-existent invitation returns not found.
    #[tokio::test]
    async fn blc_tinv_009_delete_nonexistent_not_found() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.delete_invitation_for_user(&user, "t1", "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-009: non-admin (guest) cannot delete an invitation.
    #[tokio::test]
    async fn blc_tinv_009_delete_non_admin_forbidden() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "guest")]);
        let svc = make_svc(MockTeamRepo::with(team), MockInvRepo::empty());
        let r = svc.delete_invitation_for_user(&user, "t1", "inv1").await;
        assert!(matches!(r, Err(AppError::Forbidden)));
    }

    // ── Slice 2C: accept flow ─────────────────────────────────────────────────

    /// BLC-TINV-010: accepting an invitation adds the user as a guest member.
    #[tokio::test]
    async fn blc_tinv_010_accept_new_user_becomes_guest() {
        let user = make_user("u1");
        // u1 is not a member — should be added as guest
        let team = shared_team("t1", vec![member_fetched("u2", "admin")]);
        let accept_row = inv_accept_row("inv1", team);
        let mock_team = MockTeamRepo::with(shared_team("t1", vec![member_fetched("u2", "admin")]));
        let update_called = mock_team.update_members_called.clone();
        let svc = make_svc(mock_team, MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(r.is_ok());
        assert!(*update_called.lock().unwrap(), "update_team_members must be called to add guest");
    }

    /// BLC-TINV-001: accepting an invitation for team:public returns not found.
    #[tokio::test]
    async fn blc_tinv_001_accept_public_team_not_found() {
        let user = make_user("u1");
        let accept_row = inv_accept_row("inv1", public_team_fetched());
        let svc = make_svc(MockTeamRepo::missing(), MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-001: accepting an invitation for a personal team returns not found.
    #[tokio::test]
    async fn blc_tinv_001_accept_personal_team_not_found() {
        let user = make_user("u1");
        let accept_row = inv_accept_row("inv1", personal_team("t1", "owner1"));
        let svc = make_svc(MockTeamRepo::missing(), MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-TINV-011: accepting when already content_maintainer does not downgrade the role.
    #[tokio::test]
    async fn blc_tinv_011_accept_content_maintainer_not_downgraded() {
        let user = make_user("u1");
        let team = shared_team(
            "t1",
            vec![
                member_fetched("u2", "admin"),
                member_fetched("u1", "content_maintainer"),
            ],
        );
        let accept_row = inv_accept_row("inv1", team);
        let mock_team = MockTeamRepo::with(shared_team("t1", vec![member_fetched("u2", "admin")]));
        let update_called = mock_team.update_members_called.clone();
        let svc = make_svc(mock_team, MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(r.is_ok());
        assert!(
            !*update_called.lock().unwrap(),
            "update_team_members must not downgrade content_maintainer to guest"
        );
    }

    /// BLC-TINV-011: accepting when already admin does not downgrade the role.
    #[tokio::test]
    async fn blc_tinv_011_accept_admin_not_downgraded() {
        let user = make_user("u1");
        let team = shared_team("t1", vec![member_fetched("u1", "admin")]);
        let accept_row = inv_accept_row("inv1", team);
        let mock_team = MockTeamRepo::with(shared_team("t1", vec![member_fetched("u1", "admin")]));
        let update_called = mock_team.update_members_called.clone();
        let svc = make_svc(mock_team, MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(r.is_ok());
        assert!(
            !*update_called.lock().unwrap(),
            "update_team_members must not downgrade admin to guest"
        );
    }

    /// BLC-TINV-012: accepting when already a guest does not create a duplicate member entry.
    #[tokio::test]
    async fn blc_tinv_012_accept_existing_guest_no_duplicate() {
        let user = make_user("u1");
        let team = shared_team(
            "t1",
            vec![
                member_fetched("u2", "admin"),
                member_fetched("u1", "guest"),
            ],
        );
        let accept_row = inv_accept_row("inv1", team);
        let mock_team = MockTeamRepo::with(shared_team("t1", vec![member_fetched("u2", "admin")]));
        let update_called = mock_team.update_members_called.clone();
        let svc = make_svc(mock_team, MockInvRepo::with_accept(accept_row));
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(r.is_ok());
        assert!(
            !*update_called.lock().unwrap(),
            "update_team_members must not be called when user is already a guest"
        );
    }

    /// BLC-TINV-014: accepting a non-existent invitation returns not found.
    #[tokio::test]
    async fn blc_tinv_014_accept_nonexistent_not_found() {
        let user = make_user("u1");
        let svc = make_svc(MockTeamRepo::missing(), MockInvRepo::empty());
        let r = svc.accept_invitation_for_user(&user, "inv1").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }
}
