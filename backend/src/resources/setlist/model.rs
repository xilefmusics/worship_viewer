use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use shared::setlist::{CreateSetlist, Setlist};

use crate::database::record_id_string;
use crate::resources::common::SongLinkRecord;

#[derive(Clone, Debug, Serialize, Deserialize, Default, SurrealValue)]
pub struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<RecordId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<RecordId>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl SetlistRecord {
    pub fn into_setlist(self) -> Setlist {
        Setlist {
            id: self.id.map(|r| record_id_string(&r)).unwrap_or_default(),
            owner: self.owner.map(|r| record_id_string(&r)).unwrap_or_default(),
            title: self.title,
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_payload(
        id: Option<RecordId>,
        owner: Option<RecordId>,
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

#[cfg(test)]
mod tests {
    use shared::song::Link as SongLink;

    use super::*;
    use crate::test_helpers::{seed_user, test_db};

    #[test]
    fn setlist_record_from_payload_into_setlist() {
        let id = RecordId::new("setlist", "sl1");
        let owner = RecordId::new("team", "tm");
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
