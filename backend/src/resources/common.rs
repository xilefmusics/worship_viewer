use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::player::Player;
use shared::song::{Link as SongLink, LinkOwned as SongLinkOwned, SimpleChord};

use crate::error::AppError;
use crate::resources::song::SongRecord;

/// Parse and validate a resource ID for the given table.
///
/// Accepts both plain IDs (`"abc"`) and `Thing` strings (`"setlist:abc"`).
/// Returns an error when the parsed table prefix does not match `table`.
pub(crate) fn resource_id(table: &str, id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == table {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request(format!("invalid {table} id")));
    }
    Ok((table.to_owned(), id.to_owned()))
}

/// Return `true` when `owner` is present and contained in `teams`.
pub(crate) fn belongs_to(owner: &Option<Thing>, teams: &[Thing]) -> bool {
    owner.as_ref().map(|t| teams.contains(t)).unwrap_or(false)
}

/// Coerce a string to a `song:…` [`Thing`], validating the table prefix when present.
pub(crate) fn song_thing(id: &str) -> Thing {
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "song"
    {
        return thing;
    }
    Thing::from(("song".to_owned(), id.to_owned()))
}

/// Coerce a string to a `blob:…` [`Thing`], validating the table prefix when present.
pub(crate) fn blob_thing(id: &str) -> Thing {
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "blob"
    {
        return thing;
    }
    Thing::from(("blob".to_owned(), id.to_owned()))
}

/// Build a [`Player`] from fetched song links, populating liked flags and
/// filling in default track numbers where absent.
pub(crate) fn player_from_song_links(
    liked_set: HashSet<String>,
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

/// DB record for a song reference stored on a setlist or collection.
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

/// A fully-fetched song record returned when querying setlist / collection songs via `FETCH`.
///
/// The `id` field holds the fetched [`SongRecord`]; the other fields come from the link itself.
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shared::song::Song;
    use surrealdb::sql::Thing;

    use super::*;
    use crate::error::AppError;

    #[test]
    fn resource_id_plain_id() {
        assert_eq!(
            resource_id("setlist", "abc").unwrap(),
            ("setlist".to_owned(), "abc".to_owned())
        );
    }

    #[test]
    fn resource_id_thing_string_matching_table() {
        assert_eq!(
            resource_id("setlist", "setlist:myid").unwrap(),
            ("setlist".to_owned(), "myid".to_owned())
        );
    }

    #[test]
    fn resource_id_thing_string_wrong_table() {
        let err = resource_id("setlist", "song:foo").unwrap_err();
        assert!(matches!(err, AppError::InvalidRequest(_)));
    }

    #[test]
    fn resource_id_works_for_different_tables() {
        assert_eq!(
            resource_id("song", "song:s1").unwrap(),
            ("song".to_owned(), "s1".to_owned())
        );
        assert_eq!(
            resource_id("blob", "blob:b1").unwrap(),
            ("blob".to_owned(), "b1".to_owned())
        );
    }

    #[test]
    fn belongs_to_returns_true_when_owner_in_teams() {
        let owner = Thing::from(("team".to_owned(), "t1".to_owned()));
        assert!(belongs_to(
            &Some(owner.clone()),
            &[owner, Thing::from(("team".to_owned(), "t2".to_owned()))]
        ));
    }

    #[test]
    fn belongs_to_returns_false_when_owner_missing() {
        assert!(!belongs_to(
            &None,
            &[Thing::from(("team".to_owned(), "t1".to_owned()))]
        ));
    }

    #[test]
    fn belongs_to_returns_false_when_owner_not_in_teams() {
        let owner = Thing::from(("team".to_owned(), "mine".to_owned()));
        assert!(!belongs_to(
            &Some(owner),
            &[Thing::from(("team".to_owned(), "other".to_owned()))]
        ));
    }

    #[test]
    fn player_from_song_links_sets_liked_flag_and_default_nr() {
        use shared::song::LinkOwned as SongLinkOwned;

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
}
