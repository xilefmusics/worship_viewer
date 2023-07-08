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
pub enum FileType {
    #[serde(rename(deserialize = "image/png", serialize = "image/png"))]
    PNG,
    #[serde(rename(deserialize = "image/jpeg", serialize = "image/jpeg"))]
    JPEG,
}

impl FileType {
    pub fn file_ending(&self) -> &'static str {
        match self {
            Self::PNG => ".png",
            Self::JPEG => ".jpeg",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blob {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub file_type: FileType,
    pub width: u32,
    pub height: u32,
    pub group: String,
    pub tags: Vec<String>,
}

impl Blob {
    pub fn file_name(&self) -> Result<String, AppError> {
        Ok(format!(
            "{}{}",
            self.id
                .clone()
                .ok_or(AppError::Other("blob has no id".into()))?,
            self.file_type.file_ending(),
        ))
    }
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
    pub group: String,
    pub tags: Vec<String>,
}

impl Song {
    pub fn to_player(self) -> Result<PlayerData, AppError> {
        Ok(PlayerData {
            data: self.blobs,
            toc: if self.not_a_song {
                vec![]
            } else {
                vec![TocItem {
                    idx: 0,
                    title: self.title,
                    song: self.id.ok_or(AppError::Other("song has no id".into()))?,
                }]
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionFetchedSongs {
    pub id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub title: String,
    pub songs: Vec<Song>,
    pub cover: String,
    pub group: String,
    pub tags: Vec<String>,
}

impl CollectionFetchedSongs {
    pub fn to_player(self) -> Result<PlayerData, AppError> {
        self.songs
            .into_iter()
            .map(|obj| obj.to_player())
            .try_fold(PlayerData::new(), |acc, result| Ok(acc + result?))
    }
}
