use crate::database::Database;
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

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

impl Key {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Ab => "Ab",
            Self::A => "A",
            Self::As => "A#",
            Self::Bb => "Bb",
            Self::B => "B",
            Self::Bs => "B#",
            Self::Cb => "Cb",
            Self::C => "C",
            Self::Cs => "C#",
            Self::Db => "Db",
            Self::D => "D",
            Self::Ds => "D#",
            Self::Eb => "Eb",
            Self::E => "E",
            Self::Es => "E#",
            Self::Fb => "Fb",
            Self::F => "F",
            Self::Fs => "F#",
            Self::Gb => "Gb",
            Self::G => "G",
            Self::Gs => "G#",
            Self::NotAKey => "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub nr: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<String>,
    pub collection: String,
    pub group: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SongDatabase {
    pub id: RecordId,
    pub title: String,
    pub nr: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<RecordId>,
    pub collection: RecordId,
    pub group: RecordId,
    pub tags: Vec<String>,
}

impl SongDatabase {
    pub async fn select(
        db: &Database,
        page: Option<usize>,
        page_size: Option<usize>,
        user: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(db
            .select::<Self>("song", page, page_size, user, id, None)
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>())
    }

    pub async fn select_collection(
        db: &Database,
        user: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<Song>, AppError> {
        SongCollectionWrapper::select(db, user, id).await
    }

    pub async fn create(db: &Database, songs: Vec<Song>) -> Result<Vec<Song>, AppError> {
        Ok(db
            .create_vec(
                "song",
                songs
                    .clone()
                    .into_iter()
                    .map(|song| SongDatabase::try_from(song))
                    .collect::<Result<Vec<SongDatabase>, AppError>>()?,
            )
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>())
    }
}

impl IdGetter for SongDatabase {
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

impl Into<Song> for SongDatabase {
    fn into(self) -> Song {
        Song {
            id: self.get_id_full(),
            title: self.title,
            nr: self.nr,
            key: self.key,
            language: self.language,
            title2: self.title2,
            language2: self.language2,
            not_a_song: self.not_a_song,
            blobs: self.blobs.iter().map(|blob| record2string(blob)).collect(),
            collection: record2string(&self.collection),
            group: record2string(&self.group),
            tags: self.tags,
        }
    }
}

impl TryFrom<Song> for SongDatabase {
    type Error = AppError;

    fn try_from(other: Song) -> Result<Self, Self::Error> {
        Ok(SongDatabase {
            id: string2record(&other.id)?,
            title: other.title,
            nr: other.nr,
            key: other.key,
            language: other.language,
            title2: other.title2,
            language2: other.language2,
            not_a_song: other.not_a_song,
            blobs: other
                .blobs
                .iter()
                .map(|blob| string2record(blob))
                .collect::<Result<Vec<RecordId>, AppError>>()?,
            collection: string2record(&other.collection)?,
            group: string2record(&other.group)?,
            tags: other.tags,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SongCollectionWrapper {
    pub songs: Vec<SongDatabase>,
}

impl SongCollectionWrapper {
    pub async fn select(
        db: &Database,
        user: Option<&str>,
        id: Option<&str>,
    ) -> Result<Vec<Song>, AppError> {
        Ok(db
            .select::<Self>("collection", None, None, user, id, Some("songs"))
            .await?
            .get(0)
            .ok_or(AppError::NotFound("collection not found".into()))?
            .songs
            .clone()
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>())
    }
}
