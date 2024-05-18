use crate::database::{Database, Select};
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

pub use worship_viewer_shared::types::{Key, Song};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

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
    pub async fn select<'a>(select: Select<'a>) -> Result<Vec<Song>, AppError> {
        Ok(select
            .table("song")
            .query::<Self>()
            .await?
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>())
    }

    pub async fn select_collection<'a>(select: Select<'a>) -> Result<Vec<Song>, AppError> {
        SongCollectionWrapper::select(select).await
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
    pub async fn select<'a>(select: Select<'a>) -> Result<Vec<Song>, AppError> {
        Ok(select
            .table("collection")
            .fetch("songs")
            .query::<SongCollectionWrapper>()
            .await?
            .remove(0)
            .songs
            .into_iter()
            .map(|song| song.into())
            .collect::<Vec<Song>>())
    }
}
