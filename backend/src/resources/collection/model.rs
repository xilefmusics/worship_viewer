use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::collection::{Collection, CreateCollection};
use shared::song::LinkOwned as SongLinkOwned;

use crate::resources::common::{FetchedSongRecord, SongLinkRecord, blob_thing};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CollectionRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Thing>,
    pub title: String,
    pub cover: Option<Thing>,
    #[serde(default)]
    pub songs: Vec<SongLinkRecord>,
}

impl CollectionRecord {
    pub fn into_collection(self) -> Collection {
        Collection {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            title: self.title,
            cover: self
                .cover
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_payload(
        id: Option<Thing>,
        owner: Option<Thing>,
        collection: CreateCollection,
    ) -> Self {
        Self {
            id,
            owner,
            title: collection.title,
            cover: Some(blob_thing(&collection.cover)),
            songs: collection.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct CollectionSongsRecord {
    #[serde(default)]
    pub owner: Option<Thing>,
    #[serde(default)]
    pub songs: Vec<FetchedSongRecord>,
}

impl CollectionSongsRecord {
    pub fn into_songs(self) -> Vec<SongLinkOwned> {
        self.songs
            .into_iter()
            .map(|record| record.into_song_link_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use shared::song::Link as SongLink;

    use super::*;

    #[test]
    fn collection_record_from_payload_into_collection() {
        let id = Thing::from(("collection".to_owned(), "c1".to_owned()));
        let owner = Thing::from(("team".to_owned(), "tm".to_owned()));
        let record = CollectionRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            CreateCollection {
                title: "Hits".into(),
                cover: "blob:cover1".into(),
                songs: vec![SongLink {
                    id: "s1".into(),
                    nr: None,
                    key: None,
                }],
            },
        );
        let c = record.into_collection();
        assert_eq!(c.id, "c1");
        assert_eq!(c.owner, "tm");
        assert_eq!(c.title, "Hits");
        assert_eq!(c.cover, "cover1");
        assert_eq!(c.songs.len(), 1);
        assert_eq!(c.songs[0].id, "s1");
    }
}
