use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    setlist::{CreateSetlist, Setlist},
    song::{Link as SongLink, SimpleChord, Song},
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::song::SongRecord;

pub trait Model {
    async fn get_setlists(&self, owners: Vec<String>) -> Result<Vec<Setlist>, AppError>;
    async fn get_setlist(&self, owners: Vec<String>, id: &str) -> Result<Setlist, AppError>;
    async fn get_setlist_songs(&self, owners: Vec<String>, id: &str)
    -> Result<Vec<Song>, AppError>;
    async fn create_setlist(
        &self,
        owner: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn update_setlist(
        &self,
        owners: Vec<String>,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn delete_setlist(&self, owners: Vec<String>, id: &str) -> Result<Setlist, AppError>;
}

impl Model for Database {
    async fn get_setlists(&self, owners: Vec<String>) -> Result<Vec<Setlist>, AppError> {
        let owners = owners
            .into_iter()
            .map(|owner_id| owner_thing(&owner_id))
            .collect::<Vec<_>>();

        let mut response = self
            .db
            .query("SELECT * FROM setlist WHERE owner IN $owners")
            .bind(("owners", owners))
            .await?;
        Ok(response
            .take::<Vec<SetlistRecord>>(0)?
            .into_iter()
            .map(SetlistRecord::into_setlist)
            .collect())
    }

    async fn get_setlist(&self, owners: Vec<String>, id: &str) -> Result<Setlist, AppError> {
        match self.db.select(setlist_resource(id)?).await? {
            Some(record) => {
                if setlist_belongs_to(&record, owners) {
                    Ok(record.into_setlist())
                } else {
                    Err(AppError::NotFound("setlist not found".into()))
                }
            }
            None => Err(AppError::NotFound("setlist not found".into())),
        }
    }

    async fn get_setlist_songs(
        &self,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let resource = setlist_resource(id)?;
        let mut response = self
            .db
            .query("SELECT owner, songs FROM setlist WHERE id = $id FETCH songs.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<SetlistSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))?;

        if !record.belongs_to(&owners) {
            return Err(AppError::NotFound("setlist not found".into()));
        }

        Ok(record.into_songs())
    }

    async fn create_setlist(
        &self,
        owner: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        self.db
            .create("setlist")
            .content(SetlistRecord::from_payload(
                None,
                Some(owner_thing(owner)),
                setlist,
            ))
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to create setlist"))
    }

    async fn update_setlist(
        &self,
        owners: Vec<String>,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let resource = setlist_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, owners) {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        }

        let record_id = Thing::from(resource.clone());
        let record = SetlistRecord::from_payload(Some(record_id), None, setlist);

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

    async fn delete_setlist(&self, owners: Vec<String>, id: &str) -> Result<Setlist, AppError> {
        let resource = setlist_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, owners) {
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

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, setlist: CreateSetlist) -> Self {
        Self {
            id,
            owner: owner,
            title: setlist.title,
            songs: setlist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
struct SetlistSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl SetlistSongsRecord {
    fn belongs_to(&self, owners: &[String]) -> bool {
        self.owner
            .as_ref()
            .map(|owner| owners.contains(&owner.id.to_string()))
            .unwrap_or(false)
    }

    fn into_songs(self) -> Vec<Song> {
        self.songs
            .into_iter()
            .map(|record| record.into_song())
            .collect()
    }
}

#[derive(Deserialize)]
struct FetchedSongRecord {
    #[serde(rename = "id")]
    song: SongRecord,
}

impl FetchedSongRecord {
    fn into_song(self) -> Song {
        self.song.into_song()
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

fn setlist_belongs_to(record: &SetlistRecord, owners: Vec<String>) -> bool {
    record
        .owner
        .as_ref()
        .map(|owner| owners.contains(&owner.id.to_string()))
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
