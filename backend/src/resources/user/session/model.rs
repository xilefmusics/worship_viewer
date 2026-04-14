use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use super::Session;
use crate::resources::user::UserRecord;

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
pub struct SessionIdRecord {
    pub id: Thing,
}
