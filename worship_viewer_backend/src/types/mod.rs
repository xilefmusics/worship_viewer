use surrealdb::opt::RecordId;
use surrealdb::sql::Id;

use crate::AppError;

mod blob;
mod collection;
mod group;
mod player_data;
mod song;
mod user;

pub use blob::{Blob, BlobDatabase};
pub use collection::{Collection, CollectionDatabase};
pub use group::{Group, GroupDatabase};
pub use player_data::{PlayerData, TocItem};
pub use song::{Song, SongDatabase};
pub use user::{User, UserDatabase};

pub trait IdGetter {
    fn get_id_first(&self) -> String;
    fn get_id_second(&self) -> String;
    fn get_id_full(&self) -> String;
}

pub fn string2record(str_id: &str) -> Result<RecordId, AppError> {
    let mut iter = str_id.split(":");
    Ok(RecordId {
        tb: iter
            .next()
            .ok_or(AppError::TypeConvertError("id has no table".into()))?
            .to_string(),
        id: Id::String(
            iter.next()
                .ok_or(AppError::TypeConvertError("id has no record id".into()))?
                .to_string(),
        ),
    })
}

pub fn record2string(record: &RecordId) -> String {
    format!("{}:{}", record.tb, record.id.to_string())
}
