use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

use chordlib::types::Song as SongData;
use shared::song::{CreateSong, Song, SongUserSpecificAddons};

use crate::resources::common::blob_thing;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SongRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Thing>,
    #[serde(default)]
    pub not_a_song: bool,
    #[serde(default)]
    pub blobs: Vec<Thing>,
    pub data: SongData,
    #[serde(default)]
    pub search_content: String,
}

impl SongRecord {
    pub fn into_song(self) -> Song {
        Song {
            id: self.id.map(id_from_thing).unwrap_or_default(),
            owner: self.owner.map(id_from_thing).unwrap_or_default(),
            not_a_song: self.not_a_song,
            blobs: self.blobs.into_iter().map(id_from_thing).collect(),
            data: self.data,
            user_specific_addons: SongUserSpecificAddons::default(),
        }
    }

    pub fn from_payload(id: Option<Thing>, owner: Option<Thing>, song: CreateSong) -> Self {
        let search_content = search_content_from_song_data(&song.data);
        Self {
            id,
            owner,
            not_a_song: song.not_a_song,
            blobs: song
                .blobs
                .into_iter()
                .map(|blob_id| blob_thing(&blob_id))
                .collect(),
            data: song.data,
            search_content,
        }
    }
}

pub fn search_content_from_song_data(data: &SongData) -> String {
    let mut pieces: Vec<String> = Vec::new();
    for section in &data.sections {
        for line in &section.lines {
            for part in &line.parts {
                for text in &part.languages {
                    if !text.is_empty() {
                        pieces.push(text.clone());
                    }
                }
            }
        }
    }
    pieces.join(" ")
}

pub fn id_from_thing(thing: Thing) -> String {
    id_to_plain_string(thing.id)
}

fn id_to_plain_string(id: Id) -> String {
    match id {
        Id::String(value) => value,
        Id::Number(number) => format!("{number}"),
        Id::Uuid(uuid) => uuid.to_string(),
        _ => id.to_string(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LikeRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub owner: Thing,
    pub song: Thing,
}

impl LikeRecord {
    pub fn new(owner: Thing, song: Thing) -> Self {
        Self {
            id: None,
            owner,
            song,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_record_into_song_maps_string_ids() {
        let record = SongRecord {
            id: Some(Thing::from(("song".to_owned(), "s1".to_owned()))),
            owner: Some(Thing::from(("team".to_owned(), "t9".to_owned()))),
            not_a_song: true,
            blobs: vec![Thing::from(("blob".to_owned(), "b1".to_owned()))],
            data: SongData::default(),
            search_content: String::new(),
        };
        let song = record.into_song();
        assert_eq!(song.id, "s1");
        assert_eq!(song.owner, "t9");
        assert!(song.not_a_song);
        assert_eq!(song.blobs, vec!["b1".to_string()]);
    }

    #[test]
    fn song_record_from_payload_sets_search_content_and_blob_things() {
        let data: SongData = serde_json::from_str(
            r#"{
                "titles": ["T"],
                "sections": [{
                    "title": "V",
                    "lines": [{
                        "parts": [{
                            "languages": ["Hello", "world"],
                            "comment": false
                        }]
                    }]
                }]
            }"#,
        )
        .expect("song data json");
        let create = CreateSong {
            not_a_song: false,
            blobs: vec!["blob:bb".into(), "rawblob".into()],
            data,
        };
        let record = SongRecord::from_payload(None, None, create);
        assert_eq!(record.blobs.len(), 2);
        assert_eq!(record.blobs[0].tb, "blob");
        assert_eq!(record.blobs[1].tb, "blob");
        assert_eq!(record.search_content, "Hello world");
    }

    #[test]
    fn search_content_from_song_data_empty() {
        assert_eq!(search_content_from_song_data(&SongData::default()), "");
    }

    #[test]
    fn search_content_from_song_data_joins_non_empty_languages() {
        let data: SongData = serde_json::from_str(
            r#"{
                "titles": ["T"],
                "sections": [{
                    "title": "V",
                    "lines": [{
                        "parts": [{
                            "languages": ["one", "two"],
                            "comment": false
                        }]
                    }]
                }]
            }"#,
        )
        .expect("song data json");
        assert_eq!(search_content_from_song_data(&data), "one two");
    }
}
