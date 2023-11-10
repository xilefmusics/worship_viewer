use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDatabase {
    pub id: RecordId,
    pub name: String,
    pub groups: Vec<RecordId>,
}

impl IdGetter for UserDatabase {
    fn get_id_first(&self) -> String {
        self.id.tb.clone()
    }
    fn get_id_second(&self) -> String {
        self.id.id.to_string()
    }
    fn get_id_full(&self) -> String {
        record2string(&self.id)
    }
}

impl Into<User> for UserDatabase {
    fn into(self) -> User {
        User {
            id: self.get_id_full(),
            name: self.name,
            groups: self
                .groups
                .iter()
                .map(|group| record2string(group))
                .collect(),
        }
    }
}

impl TryFrom<User> for UserDatabase {
    type Error = AppError;

    fn try_from(other: User) -> Result<Self, Self::Error> {
        Ok(UserDatabase {
            id: string2record(&other.id)?,
            name: other.name,
            groups: other
                .groups
                .iter()
                .map(|group| string2record(group))
                .collect::<Result<Vec<RecordId>, AppError>>()?,
        })
    }
}
