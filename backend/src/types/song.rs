use crate::database::{Database, Select};
use crate::error::AppError;
use crate::types::{record2string, string2record, IdGetter};

pub use worship_viewer_shared::song::{BlobSong, Key, Song, SongData, ChordSong};

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use surrealdb::opt::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlobSongDatabase {
    pub title: String,
    pub nr: String,
    pub key: Key,
    pub language: String,
    pub title2: Option<String>,
    pub language2: Option<String>,
    pub not_a_song: bool,
    pub blobs: Vec<RecordId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SongDataDatabase {
    Blob(BlobSongDatabase),
    Chord(ChordSong),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SongDatabase {
    pub id: RecordId,
    pub data: SongDataDatabase,
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
            data: self.data.into(),
            collection: record2string(&self.collection),
            group: record2string(&self.group),
            tags: self.tags,
        }
    }
}

impl Into<SongData> for SongDataDatabase {
    fn into(self) -> SongData {
        match self {
            Self::Blob(data) => SongData::Blob(data.into()),
            Self::Chord(data) => SongData::Chord(data),
        }
    }
}

impl Into<BlobSong> for BlobSongDatabase {
    fn into(self) -> BlobSong {
        BlobSong {
            title: self.title,
            nr: self.nr,
            key: self.key,
            language: self.language,
            title2: self.title2,
            language2: self.language2,
            not_a_song: self.not_a_song,
            blobs: self.blobs.iter().map(|blob| record2string(blob)).collect(),
        }
    }
}

impl TryFrom<Song> for SongDatabase {
    type Error = AppError;

    fn try_from(other: Song) -> Result<Self, Self::Error> {
        Ok(Self {
            id: string2record(&other.id)?,
            data: SongDataDatabase::try_from(other.data)?,
            collection: string2record(&other.collection)?,
            group: string2record(&other.group)?,
            tags: other.tags,
        })
    }
}

impl TryFrom<SongData> for SongDataDatabase {
    type Error = AppError;

    fn try_from(other: SongData) -> Result<Self, Self::Error> {
        Ok(match other {
            SongData::Blob(data) => Self::Blob(data.try_into()?),
            SongData::Chord(data) => Self::Chord(data),
        })
    }
}

impl TryFrom<BlobSong> for BlobSongDatabase {
    type Error = AppError;

    fn try_from(other: BlobSong) -> Result<Self, Self::Error> {
        Ok(Self {
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
