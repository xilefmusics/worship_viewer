use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::types::RecordId;

use shared::team::Team;

use crate::database::Database;
use crate::error::AppError;

use super::model::{
    DbTeamMember, TeamCreatePayload, TeamFetched, TeamIdRow, team_resource_or_reject_public,
    user_thing,
};
use super::repository::TeamRepository;

#[derive(Clone)]
pub struct SurrealTeamRepo {
    db: Arc<Database>,
}

impl SurrealTeamRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl TeamRepository for SurrealTeamRepo {
    async fn fetch_all_teams(&self) -> Result<Vec<TeamFetched>, AppError> {
        let public_thing = super::model::public_team_thing();
        Ok(self
            .inner()
            .db
            .query("SELECT * FROM team WHERE id != $public FETCH owner, members.user")
            .bind(("public", public_thing))
            .await?
            .take::<Vec<TeamFetched>>(0)?)
    }

    async fn fetch_teams_for_user(
        &self,
        user_id: &str,
        is_admin: bool,
    ) -> Result<Vec<TeamFetched>, AppError> {
        let public_thing = super::model::public_team_thing();
        let db = self.inner();
        if is_admin {
            Ok(db
                .db
                .query("SELECT * FROM team WHERE id != $public FETCH owner, members.user")
                .bind(("public", public_thing))
                .await?
                .take::<Vec<TeamFetched>>(0)?)
        } else {
            let ut = user_thing(user_id);
            Ok(db
                .db
                .query(
                    "SELECT * FROM team WHERE id != $public \
                     AND (owner = $user OR array::len(members[WHERE user = $user]) > 0) \
                     FETCH owner, members.user",
                )
                .bind(("public", public_thing))
                .bind(("user", ut))
                .await?
                .take::<Vec<TeamFetched>>(0)?)
        }
    }

    async fn fetch_team(&self, id: &str) -> Result<Option<TeamFetched>, AppError> {
        let resource = team_resource_or_reject_public(id)?;
        Ok(self
            .inner()
            .db
            .query("SELECT * FROM $tid FETCH owner, members.user")
            .bind(("tid", RecordId::new(resource.0, resource.1)))
            .await?
            .take::<Option<TeamFetched>>(0)?)
    }

    async fn create_team(&self, payload: TeamCreatePayload) -> Result<String, AppError> {
        let created: Option<TeamIdRow> = self.inner().db.create("team").content(payload).await?;
        created
            .ok_or_else(|| AppError::database("failed to create team"))
            .map(|row| crate::database::record_id_string(&row.id))
    }

    async fn update_team_name(
        &self,
        resource: (String, String),
        name: &str,
    ) -> Result<(), AppError> {
        let mut response = self
            .inner()
            .db
            .query("UPDATE $tid SET name = $name")
            .bind(("tid", RecordId::new(resource.0, resource.1)))
            .bind(("name", name.to_owned()))
            .await?;
        crate::database::surreal_take_errors("team.update_team_name", &mut response)?;
        let _ = response.check().map_err(|e| {
            crate::log_and_convert!(AppError::database, "team.update_team_name.check", e)
        })?;
        Ok(())
    }

    async fn update_team_members(
        &self,
        resource: (String, String),
        members: Vec<DbTeamMember>,
    ) -> Result<(), AppError> {
        let mut response = self
            .inner()
            .db
            .query("UPDATE $tid SET members = $members")
            .bind(("tid", RecordId::new(resource.0, resource.1)))
            .bind(("members", members))
            .await?;
        crate::database::surreal_take_errors("team.update_team_members", &mut response)?;
        let _ = response.check().map_err(|e| {
            crate::log_and_convert!(AppError::database, "team.update_team_members.check", e)
        })?;
        Ok(())
    }

    async fn delete_team_record(&self, resource: (String, String)) -> Result<(), AppError> {
        let tid = RecordId::new(resource.0, resource.1);
        let mut response = self
            .inner()
            .db
            .query("DELETE $tid")
            .bind(("tid", tid))
            .await?;
        crate::database::surreal_take_errors("team.delete_team_record", &mut response)?;
        let _ = response.check().map_err(|e| {
            crate::log_and_convert!(AppError::database, "team.delete_team_record.check", e)
        })?;
        Ok(())
    }

    async fn reassign_content(&self, from: RecordId, to: RecordId) -> Result<(), AppError> {
        for table in ["blob", "song", "collection", "setlist"] {
            let q = format!("UPDATE {table} SET owner = $to WHERE owner = $from");
            let mut response = self
                .inner()
                .db
                .query(&q)
                .bind(("to", to.clone()))
                .bind(("from", from.clone()))
                .await?;
            crate::database::surreal_take_errors("team.reassign_content", &mut response)?;
            let _ = response.check().map_err(|e| {
                crate::log_and_convert!(AppError::database, "team.reassign_content.check", e)
            })?;
        }
        Ok(())
    }

    async fn load_team_display(&self, id: &str) -> Result<Team, AppError> {
        let resource = team_resource_or_reject_public(id)?;
        let row = self
            .inner()
            .db
            .query("SELECT * FROM $tid FETCH owner, members.user")
            .bind(("tid", RecordId::new(resource.0, resource.1)))
            .await?
            .take::<Option<TeamFetched>>(0)?
            .ok_or_else(|| AppError::NotFound("team not found".into()))?;
        row.into_team()
    }
}
