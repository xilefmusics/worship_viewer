use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{setlist::Setlist, song::Link as SongLink};

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_setlists(&self) -> Result<Vec<Setlist>, AppError>;
    async fn get_setlist(&self, id: &str) -> Result<Setlist, AppError>;
    async fn create_setlist(&self, setlist: Setlist) -> Result<Setlist, AppError>;
    async fn update_setlist(&self, id: &str, setlist: Setlist) -> Result<Setlist, AppError>;
    async fn delete_setlist(&self, id: &str) -> Result<Setlist, AppError>;
}

impl Model for Database {
    async fn get_setlists(&self) -> Result<Vec<Setlist>, AppError> {
        Ok(self
            .db
            .select("setlist")
            .await?
            .into_iter()
            .map(SetlistRecord::into_setlist)
            .collect())
    }

    async fn get_setlist(&self, id: &str) -> Result<Setlist, AppError> {
        self.db
            .select(setlist_resource(id)?)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }

    async fn create_setlist(&self, setlist: Setlist) -> Result<Setlist, AppError> {
        self.db
            .create("setlist")
            .content(SetlistRecord::from_setlist(setlist))
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to create setlist"))
    }

    async fn update_setlist(&self, id: &str, setlist: Setlist) -> Result<Setlist, AppError> {
        self.db
            .update(setlist_resource(id)?)
            .content(SetlistRecord::from_setlist(setlist))
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }

    async fn delete_setlist(&self, id: &str) -> Result<Setlist, AppError> {
        self.db
            .delete(setlist_resource(id)?)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }
}

fn setlist_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "setlist" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid setlist id"));
    }

    Ok(("setlist".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLink>,
}

impl SetlistRecord {
    fn into_setlist(self) -> Setlist {
        Setlist {
            id: self.id.map(|thing| thing.id.to_string()),
            title: self.title,
            songs: self.songs,
        }
    }

    fn from_setlist(setlist: Setlist) -> Self {
        Self {
            id: setlist
                .id
                .filter(|id| !id.is_empty())
                .map(|id| Thing::from(("setlist".to_owned(), id))),
            title: setlist.title,
            songs: setlist.songs,
        }
    }
}
