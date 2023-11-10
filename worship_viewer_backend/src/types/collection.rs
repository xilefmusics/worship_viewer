use crate::database::Database;
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub songs: Vec<String>,
    pub cover: String,
    pub group: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionDatabase {
    pub id: RecordId,
    pub title: String,
    pub songs: Vec<RecordId>,
    pub cover: RecordId,
    pub group: RecordId,
    pub tags: Vec<String>,
}

impl CollectionDatabase {
    pub async fn select(
        db: &Database,
        page: Option<usize>,
        page_size: Option<usize>,
        user: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .select::<Self>("collection", page, page_size, user, id)
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Collection>>())
    }
}

impl IdGetter for CollectionDatabase {
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

impl Into<Collection> for CollectionDatabase {
    fn into(self) -> Collection {
        Collection {
            id: self.get_id_full(),
            title: self.title,
            songs: self.songs.iter().map(|song| record2string(song)).collect(),
            cover: record2string(&self.cover),
            group: record2string(&self.group),
            tags: self.tags,
        }
    }
}

impl TryFrom<Collection> for CollectionDatabase {
    type Error = AppError;

    fn try_from(other: Collection) -> Result<Self, Self::Error> {
        Ok(CollectionDatabase {
            id: string2record(&other.id)?,
            title: other.title,
            songs: other
                .songs
                .iter()
                .map(|song| string2record(song))
                .collect::<Result<Vec<RecordId>, AppError>>()?,
            cover: string2record(&other.cover)?,
            group: string2record(&other.group)?,
            tags: other.tags,
        })
    }
}
