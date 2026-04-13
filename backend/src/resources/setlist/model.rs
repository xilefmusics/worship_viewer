use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    player::Player,
    setlist::{CreateSetlist, Setlist},
    song::{Link as SongLink, LinkOwned as SongLinkOwned, SimpleChord},
};

use crate::error::AppError;
use crate::resources::song::SongRecord;

pub(crate) fn player_from_song_links(
    liked_set: std::collections::HashSet<String>,
    links: Vec<SongLinkOwned>,
) -> Result<Player, AppError> {
    links
        .into_iter()
        .enumerate()
        .map(|(idx, link)| {
            Player::from(SongLinkOwned {
                liked: liked_set.contains(&link.song.id),
                song: link.song,
                nr: Some(link.nr.unwrap_or_else(|| (idx + 1).to_string())),
                key: link.key,
            })
        })
        .try_fold(Player::default(), |acc, player| {
            Ok::<Player, AppError>(acc + player)
        })
}

pub(crate) fn setlist_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "setlist" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid setlist id"));
    }

    Ok(("setlist".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) owner: Option<Thing>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl SetlistRecord {
    pub(crate) fn into_setlist(self) -> Setlist {
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

    pub(crate) fn from_payload(
        id: Option<Thing>,
        owner: Option<Thing>,
        setlist: CreateSetlist,
    ) -> Self {
        Self {
            id,
            owner,
            title: setlist.title,
            songs: setlist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct SetlistSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl SetlistSongsRecord {
    pub(crate) fn belongs_to(&self, teams: &[Thing]) -> bool {
        self.owner
            .as_ref()
            .map(|t| teams.contains(t))
            .unwrap_or(false)
    }

    pub(crate) fn into_songs(self) -> Vec<SongLinkOwned> {
        self.songs
            .into_iter()
            .map(|record| record.into_song_link_owned())
            .collect()
    }
}

#[derive(Deserialize)]
pub(crate) struct FetchedSongRecord {
    #[serde(rename = "id")]
    song: SongRecord,
    #[serde(default)]
    nr: Option<String>,
    #[serde(default)]
    key: Option<SimpleChord>,
    #[serde(default)]
    liked: bool,
}

impl FetchedSongRecord {
    pub(crate) fn into_song_link_owned(self) -> SongLinkOwned {
        SongLinkOwned {
            song: self.song.into_song(),
            nr: self.nr,
            key: self.key,
            liked: self.liked,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SongLinkRecord {
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

pub(crate) fn setlist_belongs_to(record: &SetlistRecord, teams: &[Thing]) -> bool {
    record
        .owner
        .as_ref()
        .map(|t| teams.contains(t))
        .unwrap_or(false)
}

fn song_thing(id: &str) -> Thing {
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "song"
    {
        return thing;
    }

    Thing::from(("song".to_owned(), id.to_owned()))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shared::song::Song;

    use super::*;
    use crate::test_helpers::{seed_user, test_db};

    #[test]
    fn setlist_resource_plain_id() {
        assert_eq!(
            setlist_resource("abc").unwrap(),
            ("setlist".to_owned(), "abc".to_owned())
        );
    }

    #[test]
    fn setlist_resource_thing_string() {
        let id = "setlist:myid";
        assert_eq!(
            setlist_resource(id).unwrap(),
            ("setlist".to_owned(), "myid".to_owned())
        );
    }

    #[test]
    fn setlist_resource_wrong_table_is_invalid() {
        let err = setlist_resource("song:foo").unwrap_err();
        assert!(matches!(err, AppError::InvalidRequest(_)));
    }

    #[test]
    fn setlist_belongs_to_when_owner_in_teams() {
        let owner = Thing::from(("team".to_owned(), "t1".to_owned()));
        let record = SetlistRecord {
            id: None,
            owner: Some(owner.clone()),
            title: "x".into(),
            songs: vec![],
        };
        assert!(setlist_belongs_to(
            &record,
            &[owner, Thing::from(("team".to_owned(), "t2".to_owned()))]
        ));
    }

    #[test]
    fn setlist_belongs_to_false_when_owner_missing() {
        let record = SetlistRecord {
            id: None,
            owner: None,
            title: "x".into(),
            songs: vec![],
        };
        assert!(!setlist_belongs_to(
            &record,
            &[Thing::from(("team".to_owned(), "t1".to_owned()))]
        ));
    }

    #[test]
    fn setlist_belongs_to_false_when_not_in_teams() {
        let owner = Thing::from(("team".to_owned(), "mine".to_owned()));
        let record = SetlistRecord {
            id: None,
            owner: Some(owner),
            title: "x".into(),
            songs: vec![],
        };
        assert!(!setlist_belongs_to(
            &record,
            &[Thing::from(("team".to_owned(), "other".to_owned()))]
        ));
    }

    #[test]
    fn setlist_record_from_payload_into_setlist() {
        let id = Thing::from(("setlist".to_owned(), "sl1".to_owned()));
        let owner = Thing::from(("team".to_owned(), "tm".to_owned()));
        let record = SetlistRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            CreateSetlist {
                title: "Sunday".into(),
                songs: vec![SongLink {
                    id: "s1".into(),
                    nr: Some("1".into()),
                    key: None,
                }],
            },
        );
        let setlist = record.into_setlist();
        assert_eq!(setlist.id, "sl1");
        assert_eq!(setlist.owner, "tm");
        assert_eq!(setlist.title, "Sunday");
        assert_eq!(setlist.songs.len(), 1);
        assert_eq!(setlist.songs[0].id, "s1");
    }

    #[test]
    fn player_from_song_links_sets_liked_and_default_nr() {
        let mut liked = HashSet::new();
        liked.insert("a".into());
        let s1 = Song {
            id: "a".into(),
            ..Default::default()
        };
        let s2 = Song {
            id: "b".into(),
            ..Default::default()
        };
        let links = vec![
            SongLinkOwned {
                song: s1,
                nr: None,
                key: None,
                liked: false,
            },
            SongLinkOwned {
                song: s2,
                nr: Some("x".into()),
                key: None,
                liked: false,
            },
        ];
        let player = player_from_song_links(liked, links).unwrap();
        assert!(player.is_liked("a"));
        assert!(!player.is_liked("b"));
        let toc = player.toc();
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].nr, "1");
        assert_eq!(toc[1].nr, "x");
    }

    #[tokio::test]
    async fn smoke_create_and_read_setlist() {
        use actix_web::web::Data;

        use crate::resources::setlist::{SetlistService, SurrealSetlistRepo};
        use crate::resources::team::SurrealTeamResolver;

        let db = test_db().await.expect("test db");
        let data = Data::from(db.clone());
        let svc = SetlistService::new(
            SurrealSetlistRepo::new(data.clone()),
            SurrealTeamResolver::new(data.clone()),
            data.clone(),
        );
        let user = seed_user(&db).await.expect("seed user");
        let created = svc
            .create_setlist_for_user(
                &user,
                CreateSetlist {
                    title: "Smoke".to_string(),
                    songs: vec![],
                },
            )
            .await
            .expect("create setlist");
        let fetched = svc
            .get_setlist_for_user(&user, &created.id)
            .await
            .expect("get setlist");
        assert_eq!(fetched.title, "Smoke");
        assert_eq!(fetched.id, created.id);
        assert!(fetched.songs.is_empty());
    }
}
