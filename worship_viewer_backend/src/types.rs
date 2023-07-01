use super::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use worship_viewer_shared::types::{Collection, PlayerData, TocItem};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupWrapper<T> {
    pub group: String,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserGroupsFetched {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub name: String,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserGroupsId {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub name: String,
    pub groups: Vec<String>,
}

impl UserGroupsId {
    pub fn drop_groups(&self) -> User {
        User {
            id: self.id.clone(),
            created: self.created.clone(),
            name: self.name.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub width: u32,
    pub height: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Key {
    Ab,
    A,
    #[serde(rename(deserialize = "A#", serialize = "A#"))]
    As,
    Bb,
    B,
    #[serde(rename(deserialize = "B#", serialize = "B#"))]
    Bs,
    Cb,
    C,
    #[serde(rename(deserialize = "C#", serialize = "C#"))]
    Cs,
    Db,
    D,
    #[serde(rename(deserialize = "D#", serialize = "D#"))]
    Ds,
    Eb,
    E,
    #[serde(rename(deserialize = "E#", serialize = "E#"))]
    Es,
    Fb,
    F,
    #[serde(rename(deserialize = "F#", serialize = "F#"))]
    Fs,
    Gb,
    G,
    #[serde(rename(deserialize = "G#", serialize = "G#"))]
    Gs,
    #[serde(rename(deserialize = "", serialize = ""))]
    NotAKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub title: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    pub tags: Vec<String>,
}

impl Song {
    pub fn drop_blobs(self) -> SongDatabase {
        SongDatabase {
            id: self.id,
            created: self.created,
            title: self.title,
            key: self.key,
            language: self.language,
            title2: self.title2,
            language2: self.language2,
            not_a_song: self.not_a_song,
            tags: self.tags,
        }
    }

    pub fn to_player(self) -> Result<PlayerData, AppError> {
        Ok(PlayerData {
            data: self.blobs,
            toc: vec![TocItem {
                idx: 0,
                title: self.title,
                song: self.id.ok_or(AppError::Other("song has no id".into()))?,
            }],
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SongDatabase {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub title: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionSongsBlobsFetched {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub title: String,
    pub songs: Vec<Song>,
    pub tags: Vec<String>,
}

impl CollectionSongsBlobsFetched {
    pub fn to_player(self) -> Result<PlayerData, AppError> {
        self.songs
            .into_iter()
            .map(|song| song.to_player())
            .try_fold(PlayerData::new(), |acc, result| Ok(acc + result?))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionDatabase {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub title: String,
    pub cover: String,
    pub tags: Vec<String>,
}

impl CollectionDatabase {
    pub fn from_collection(collection: Collection) -> Self {
        CollectionDatabase {
            id: collection.id,
            created: collection.created,
            title: collection.title,
            cover: collection.cover,
            tags: collection.tags,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TitleAndSongAndBlobs {
    title: Option<String>,
    song: String,
    blobs: Vec<String>,
}

impl TitleAndSongAndBlobs {
    pub fn to_player(self) -> Result<PlayerData, AppError> {
        Ok(PlayerData {
            data: self.blobs,
            toc: if self.title.is_some() && self.title.clone().unwrap().len() > 0 {
                vec![TocItem {
                    idx: 0,
                    title: self.title.unwrap(),
                    song: self.song,
                }]
            } else {
                vec![]
            },
        })
    }
}
