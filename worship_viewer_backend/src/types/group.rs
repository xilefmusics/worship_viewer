use crate::database::Database;
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

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

impl GroupDatabase {
    pub async fn select(
        db: &Database,
        page: Option<usize>,
        page_size: Option<usize>,
        user: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<Group>, AppError> {
        Ok(db
            .select::<Self>("group", page, page_size, user, id)
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<Group>>())
    }

    pub async fn create(db: &Database, groups: Vec<Group>) -> Result<Vec<Group>, AppError> {
        Ok(db
            .create_vec(
                "group",
                groups
                    .clone()
                    .into_iter()
                    .map(|group| GroupDatabase::try_from(group))
                    .collect::<Result<Vec<GroupDatabase>, AppError>>()?,
            )
            .await?
            .into_iter()
            .map(|group| group.into())
            .collect::<Vec<Group>>())
    }
}

impl IdGetter for GroupDatabase {
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
        Ok(GroupDatabase {
            id: string2record(&other.id)?,
            name: other.name,
        })
    }
}
