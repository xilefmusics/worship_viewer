use crate::error::AppError;
use crate::types::IdGetter;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;
use surrealdb::sql::Id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupDatabase {
    pub id: RecordId,
    pub name: String,
}

impl IdGetter for GroupDatabase {
    fn get_id_first(&self) -> String {
        self.id.tb.clone()
    }
    fn get_id_second(&self) -> String {
        self.id.id.to_string()
    }
    fn get_id_full(&self) -> String {
        format!("{}:{}", self.get_id_first(), self.get_id_second())
    }
}

impl Into<Group> for GroupDatabase {
    fn into(self) -> Group {
        Group {
            id: self.get_id_full(),
            name: self.name,
        }
    }
}

impl TryFrom<Group> for GroupDatabase {
    type Error = AppError;

    fn try_from(other: Group) -> Result<Self, Self::Error> {
        let mut iter = other.id.split(":");
        Ok(GroupDatabase {
            id: RecordId {
                tb: iter
                    .next()
                    .ok_or(AppError::TypeConvertError("id has no table".into()))?
                    .to_string(),
                id: Id::String(
                    iter.next()
                        .ok_or(AppError::TypeConvertError("id has no record id".into()))?
                        .to_string(),
                ),
            },
            name: other.name,
        })
    }
}