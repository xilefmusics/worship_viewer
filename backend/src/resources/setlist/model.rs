use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::setlist::{CreateSetlist, Setlist};

use crate::resources::common::{FetchedSongRecord, SongLinkRecord, belongs_to};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Thing>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl SetlistRecord {
    pub fn into_setlist(self) -> Setlist {
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

    pub fn from_payload(id: Option<Thing>, owner: Option<Thing>, setlist: CreateSetlist) -> Self {
        Self {
            id,
            owner,
            title: setlist.title,
            songs: setlist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct SetlistSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl SetlistSongsRecord {
    pub fn belongs_to(&self, teams: &[Thing]) -> bool {
        belongs_to(&self.owner, teams)
    }

    pub fn into_songs(self) -> Vec<shared::song::LinkOwned> {
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
    use crate::test_helpers::{seed_user, test_db};

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

    #[tokio::test]
    async fn smoke_create_and_read_setlist() {
        use crate::resources::setlist::{SetlistService, SurrealSetlistRepo};
        use crate::resources::team::{SurrealTeamResolver, UserPermissions};

        let db = test_db().await.expect("test db");
        let svc = SetlistService::new(
            SurrealSetlistRepo::new(db.clone()),
            std::sync::Arc::new(SurrealTeamResolver::new(db.clone())),
            db.clone(),
        );
        let user = seed_user(&db).await.expect("seed user");
        let perms = UserPermissions::from_ref(&user, &svc.teams);
        let created = svc
            .create_setlist_for_user(
                &perms,
                CreateSetlist {
                    title: "Smoke".to_string(),
                    songs: vec![],
                },
            )
            .await
            .expect("create setlist");
        let fetched = svc
            .get_setlist_for_user(&perms, &created.id)
            .await
            .expect("get setlist");
        assert_eq!(fetched.title, "Smoke");
        assert_eq!(fetched.id, created.id);
        assert!(fetched.songs.is_empty());
    }
}
