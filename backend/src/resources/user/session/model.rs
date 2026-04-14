use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use super::Session;
use crate::database::Database;
use crate::error::AppError;
use crate::resources::user::{Model as UserDbModel, UserRecord};

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

impl Database {
    pub async fn create_session_for_user_by_id(
        &self,
        user_id: &str,
        ttl_seconds: i64,
    ) -> Result<Session, AppError> {
        self.create_session(Session::new(
            UserDbModel::get_user(self, user_id).await?,
            ttl_seconds,
        ))
        .await
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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::resources::{Session, SessionModel, User, UserModel, UserRole};
    use crate::test_helpers::{create_user, test_db};

    #[tokio::test]
    async fn blc_user_admin_and_session_lifecycle() {
        let db = test_db().await.expect("db");
        let admin = db
            .create_user(User {
                id: String::new(),
                email: "admin-sess@test.local".into(),
                role: UserRole::Admin,
                default_collection: None,
                created_at: Utc::now(),
                last_login_at: None,
                request_count: 0,
            })
            .await
            .expect("admin");

        let u = create_user(&db, "user-sess@test.local")
            .await
            .expect("user");

        let sess = db
            .create_session(Session::new(u.clone(), 3600))
            .await
            .expect("session");
        assert!(!sess.id.is_empty());

        let got = db.get_session(&sess.id).await.expect("get session");
        assert_eq!(got.user.id, u.id);

        let by_user = db.get_sessions_by_user_id(&u.id).await.expect("by user");
        assert!(by_user.iter().any(|s| s.id == sess.id));

        db.delete_session(&sess.id).await.expect("delete session");
        let _ = admin;
    }
}
