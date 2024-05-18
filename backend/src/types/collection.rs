use crate::database::{Database, Select};
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

pub use worship_viewer_shared::types::Collection;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

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
    pub async fn select<'a>(select: Select<'a>) -> Result<Vec<Collection>, AppError> {
        Ok(select
            .table("collection")
            .query::<Self>()
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<Collection>>())
    }

    pub async fn create(
        db: &Database,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .create_vec(
                "collection",
                collections
                    .clone()
                    .into_iter()
                    .map(|collection| CollectionDatabase::try_from(collection))
                    .collect::<Result<Vec<CollectionDatabase>, AppError>>()?,
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
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
