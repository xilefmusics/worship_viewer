use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    setlist::{CreateSetlist, Setlist},
    song::{Link as SongLink, SimpleChord},
};

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn get_setlists(&self, owner_id: &str) -> Result<Vec<Setlist>, AppError>;
    async fn get_setlist(&self, owner_id: &str, id: &str) -> Result<Setlist, AppError>;
    async fn create_setlist(
        &self,
        owner_id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn update_setlist(
        &self,
        owner_id: &str,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn delete_setlist(&self, owner_id: &str, id: &str) -> Result<Setlist, AppError>;
}

impl Model for Database {
    async fn get_setlists(&self, owner_id: &str) -> Result<Vec<Setlist>, AppError> {
        let mut response = self
            .db
            .query("SELECT * FROM setlist WHERE owner = $owner")
            .bind(("owner", owner_thing(owner_id)))
            .await?;
        Ok(response
            .take::<Vec<SetlistRecord>>(0)?
            .into_iter()
            .map(SetlistRecord::into_setlist)
            .collect())
    }

    async fn get_setlist(&self, owner_id: &str, id: &str) -> Result<Setlist, AppError> {
        match self.db.select(setlist_resource(id)?).await? {
            Some(record) => {
                if setlist_belongs_to(&record, owner_id) {
                    Ok(record.into_setlist())
                } else {
                    Err(AppError::NotFound("setlist not found".into()))
                }
            }
            None => Err(AppError::NotFound("setlist not found".into())),
        }
    }

    async fn create_setlist(
        &self,
        owner_id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        self.db
            .create("setlist")
            .content(SetlistRecord::from_payload(
                None,
                owner_thing(owner_id),
                setlist,
            ))
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to create setlist"))
    }

    async fn update_setlist(
        &self,
        owner_id: &str,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let resource = setlist_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, owner_id) {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        }

        let record_id = Thing::from(resource.clone());
        let record = SetlistRecord::from_payload(Some(record_id), owner_thing(owner_id), setlist);

        if let Some(updated) = self
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(SetlistRecord::into_setlist)
        {
            return Ok(updated);
        }

        self.db
            .create(resource)
            .content(record)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to upsert setlist"))
    }

    async fn delete_setlist(&self, owner_id: &str, id: &str) -> Result<Setlist, AppError> {
        let resource = setlist_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, owner_id) {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        } else {
            return Err(AppError::NotFound("setlist not found".into()));
        }

        self.db
            .delete(resource)
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl SetlistRecord {
    fn into_setlist(self) -> Setlist {
        Setlist {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            title: self.title,
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Thing, setlist: CreateSetlist) -> Self {
        Self {
            id,
            owner: Some(owner),
            title: setlist.title,
            songs: setlist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SongLinkRecord {
    id: Thing,
    #[serde(default)]
    nr: Option<String>,
    #[serde(default)]
    key: Option<SimpleChord>,
}

impl From<SongLinkRecord> for SongLink {
    fn from(record: SongLinkRecord) -> Self {
        Self {
            id: record.id.id.to_string(),
            nr: record.nr,
            key: record.key,
        }
    }
}

impl From<SongLink> for SongLinkRecord {
    fn from(link: SongLink) -> Self {
        Self {
            id: song_thing(&link.id),
            nr: link.nr,
            key: link.key,
        }
    }
}

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

fn setlist_belongs_to(record: &SetlistRecord, owner_id: &str) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owner.id.to_string() == owner_id)
        .unwrap_or(false)
}

fn song_thing(id: &str) -> Thing {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "song" {
            return thing;
        }
    }

    Thing::from(("song".to_owned(), id.to_owned()))
}
