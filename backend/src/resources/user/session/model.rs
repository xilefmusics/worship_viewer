use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use super::Session;
use crate::database::Database;
use crate::error::AppError;
use crate::resources::user::UserRecord;

pub trait Model {
    async fn get_sessions_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, AppError>;
    async fn get_session(&self, id: &str) -> Result<Session, AppError>;
    async fn create_session(&self, session: Session) -> Result<Session, AppError>;
    async fn delete_session(&self, id: &str) -> Result<Session, AppError>;
    async fn get_session_and_update_user_metrics_or_delete_if_exipired(
        &self,
        id: &str,
    ) -> Result<Option<Session>, AppError>;
}

impl Model for Database {
    async fn get_session(&self, id: &str) -> Result<Session, AppError> {
        self.db
            .query("SELECT * FROM session WHERE id = $id FETCH user")
            .bind(("id", Thing::from(("session".to_owned(), id.to_string()))))
            .await?
            .take::<Option<SessionRecord>>(0)?
            .map(SessionRecord::into_session)
            .ok_or(AppError::NotFound("session not found".into()))
    }

    async fn create_session(&self, session: Session) -> Result<Session, AppError> {
        let session: SessionIdRecord = self
            .db
            .create(("session", session.id.clone()))
            .content(SessionCreateRecord::from_session(session))
            .await?
            .ok_or_else(|| AppError::database("Failed to create session"))?;

        self.get_session(&session.id.id.to_raw()).await
    }

    async fn delete_session(&self, id: &str) -> Result<Session, AppError> {
        let session = self.get_session(id).await?;
        let _: Option<SessionIdRecord> = self.db.delete(("session", id)).await?;
        Ok(session)
    }

    async fn get_session_and_update_user_metrics_or_delete_if_exipired(
        &self,
        id: &str,
    ) -> Result<Option<Session>, AppError> {
        Ok(self
            .db
            .query(
                r#"
            LET $sid = type::thing("session", $id);
                        
            DELETE $sid
            WHERE expires_at != NONE
              AND expires_at <= time::now();
                        
            UPDATE user
            SET
              last_login_at = time::now(),
              request_count += 1
            WHERE id = (SELECT user FROM $sid)[0].user;
                        
            RETURN (SELECT * FROM $sid FETCH user)[0];
                "#,
            )
            .bind(("id", id.to_owned()))
            .await
            .map_err(AppError::database)?
            .take::<Option<SessionRecord>>(3)
            .map_err(AppError::database)?
            .map(SessionRecord::into_session))
    }

    async fn get_sessions_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, AppError> {
        Ok(self
            .db
            .query("SELECT * FROM session WHERE user = $user FETCH user")
            .bind(("user", Thing::from(("user".to_owned(), user_id.to_owned()))))
            .await?
            .take::<Vec<SessionRecord>>(0)?
            .into_iter()
            .map(|record| record.into_session())
            .collect())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionRecord {
    pub id: Thing,
    pub user: UserRecord,
    pub created_at: Datetime,
    pub expires_at: Datetime,
}

impl SessionRecord {
    pub fn into_session(self) -> Session {
        Session {
            id: self.id.id.to_raw(),
            user: self.user.into_user(),
            created_at: self.created_at.into(),
            expires_at: self.expires_at.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionCreateRecord {
    pub user: Thing,
    pub expires_at: Datetime,
}

impl SessionCreateRecord {
    pub fn from_session(session: Session) -> Self {
        Self {
            user: Thing::from(("user".to_owned(), session.user.id)),
            expires_at: session.expires_at.into(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct SessionIdRecord {
    id: Thing,
}
