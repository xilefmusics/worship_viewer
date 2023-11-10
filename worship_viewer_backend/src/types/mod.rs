use surrealdb::opt::RecordId;
use surrealdb::sql::Id;

use crate::AppError;

mod blob;
mod group;
mod user;

pub use blob::{Blob, BlobDatabase};
pub use group::{Group, GroupDatabase};
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
