use actix_web::{HttpResponse, web::Data};
use shared::api::ListQuery;
use shared::player::Player;
use shared::setlist::{CreateSetlist, Setlist};
use shared::song::Song;

use crate::error::AppError;
use crate::resources::song::LikedSongIds;
use crate::resources::song::{Format, export};
use crate::resources::team::{TeamResolver, UserPermissions};

use crate::resources::common::player_from_song_links;
use super::repository::SetlistRepository;

/// Application service: team resolution, authorization, and orchestration for setlists.
#[derive(Clone)]
pub struct SetlistService<R, T, L> {
    pub repo: R,
    pub teams: T,
    pub likes: L,
}

impl<R, T, L> SetlistService<R, T, L> {
    pub fn new(repo: R, teams: T, likes: L) -> Self {
        Self { repo, teams, likes }
    }
}

impl<R: SetlistRepository, T: TeamResolver, L: LikedSongIds> SetlistService<R, T, L> {
    pub async fn list_setlists_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError> {
        let read_teams = perms.read_teams().await?;
        self.repo.get_setlists(read_teams, pagination).await
    }

    pub async fn get_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Setlist, AppError> {
        let read_teams = perms.read_teams().await?;
        self.repo.get_setlist(read_teams, id).await
    }

    pub async fn setlist_player_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Player, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user_id),
            perms.read_teams()
        )?;
        let links = self.repo.get_setlist_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn export_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = perms.read_teams().await?;
        let songs: Vec<Song> = self
            .repo
            .get_setlist_songs(read_teams, id)
            .await?
            .into_iter()
            .map(|l| l.song)
            .collect();
        export(songs, format).await
    }

    pub async fn setlist_songs_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user_id),
            perms.read_teams()
        )?;
        Ok(self
            .repo
            .get_setlist_songs(read_teams, id)
            .await?
            .into_iter()
            .map(|song_link_owned| {
                let mut song = song_link_owned.song;
                song.user_specific_addons.liked = liked_set.contains(&song.id);
                song
            })
            .collect())
    }

    pub async fn create_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        self.repo.create_setlist(&perms.user().id, setlist).await
    }

    pub async fn update_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo.update_setlist(write_teams, id, setlist).await
    }

    pub async fn delete_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Setlist, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo.delete_setlist(write_teams, id).await
    }
}

/// Type alias for the production HTTP stack.
pub type SetlistServiceHandle = SetlistService<
    super::surreal_repo::SurrealSetlistRepo,
    crate::resources::team::SurrealTeamResolver,
    Data<crate::database::Database>,
>;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_trait::async_trait;
    use surrealdb::sql::Thing;

    use shared::api::ListQuery;
    use shared::setlist::{CreateSetlist, Setlist};
    use shared::song::LinkOwned as SongLinkOwned;

    use crate::error::AppError;
    use crate::resources::User;
    use crate::resources::song::LikedSongIds;
    use crate::resources::team::{TeamResolver, UserPermissions};

    use super::{SetlistRepository, SetlistService};

    struct MockRepo {
        setlists: Vec<Setlist>,
        get_returns: Option<Setlist>,
        update_ok: bool,
    }

    #[async_trait]
    impl SetlistRepository for MockRepo {
        async fn get_setlists(
            &self,
            _read_teams: Vec<Thing>,
            _pagination: ListQuery,
        ) -> Result<Vec<Setlist>, AppError> {
            Ok(self.setlists.clone())
        }

        async fn get_setlist(
            &self,
            _read_teams: Vec<Thing>,
            _id: &str,
        ) -> Result<Setlist, AppError> {
            self.get_returns
                .clone()
                .ok_or_else(|| AppError::NotFound("setlist not found".into()))
        }

        async fn get_setlist_songs(
            &self,
            _read_teams: Vec<Thing>,
            _id: &str,
        ) -> Result<Vec<SongLinkOwned>, AppError> {
            Ok(vec![])
        }

        async fn create_setlist(
            &self,
            _owner: &str,
            _setlist: CreateSetlist,
        ) -> Result<Setlist, AppError> {
            unreachable!("not used in these tests")
        }

        async fn update_setlist(
            &self,
            _write_teams: Vec<Thing>,
            _id: &str,
            _setlist: CreateSetlist,
        ) -> Result<Setlist, AppError> {
            if self.update_ok {
                Ok(Setlist {
                    id: "x".into(),
                    owner: "t".into(),
                    title: "ok".into(),
                    songs: vec![],
                })
            } else {
                Err(AppError::NotFound("setlist not found".into()))
            }
        }

        async fn delete_setlist(
            &self,
            _write_teams: Vec<Thing>,
            _id: &str,
        ) -> Result<Setlist, AppError> {
            Err(AppError::NotFound("setlist not found".into()))
        }
    }

    struct MockTeams {
        read: Vec<Thing>,
        write: Vec<Thing>,
    }

    #[async_trait]
    impl TeamResolver for MockTeams {
        async fn content_read_teams(&self, _user: &User) -> Result<Vec<Thing>, AppError> {
            Ok(self.read.clone())
        }

        async fn content_write_teams(&self, _user: &User) -> Result<Vec<Thing>, AppError> {
            Ok(self.write.clone())
        }

        async fn personal_team(&self, _user_id: &str) -> Result<Thing, AppError> {
            Err(AppError::database("unused"))
        }
    }

    struct MockLikes {
        ids: HashSet<String>,
    }

    #[async_trait]
    impl LikedSongIds for MockLikes {
        async fn liked_song_ids(&self, _user_id: &str) -> Result<HashSet<String>, AppError> {
            Ok(self.ids.clone())
        }
    }

    fn team_a() -> Thing {
        Thing::from(("team".to_owned(), "a".to_owned()))
    }

    fn team_b() -> Thing {
        Thing::from(("team".to_owned(), "b".to_owned()))
    }

    fn test_user() -> User {
        User::new("u@test.local")
    }

    /// BLC-SETL-006: missing setlist → NotFound
    #[tokio::test]
    async fn get_returns_not_found_when_setlist_missing() {
        let user = test_user();
        let svc = SetlistService::new(
            MockRepo {
                setlists: vec![],
                get_returns: None,
                update_ok: false,
            },
            MockTeams {
                read: vec![team_a()],
                write: vec![],
            },
            MockLikes {
                ids: HashSet::new(),
            },
        );
        let perms = UserPermissions::new(&user, &svc.teams);
        let r = svc.get_setlist_for_user(&perms, "nope").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-007: write teams exclude owner → update NotFound
    #[tokio::test]
    async fn update_rejects_when_user_not_in_write_teams() {
        let user = test_user();
        let svc = SetlistService::new(
            MockRepo {
                setlists: vec![],
                get_returns: None,
                update_ok: false,
            },
            MockTeams {
                read: vec![team_a()],
                write: vec![team_b()],
            },
            MockLikes {
                ids: HashSet::new(),
            },
        );
        let perms = UserPermissions::new(&user, &svc.teams);
        let r = svc
            .update_setlist_for_user(
                &perms,
                "id",
                CreateSetlist {
                    title: "t".into(),
                    songs: vec![],
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-008: owner can update (repo succeeds)
    #[tokio::test]
    async fn update_succeeds_for_owner() {
        let user = test_user();
        let svc = SetlistService::new(
            MockRepo {
                setlists: vec![],
                get_returns: None,
                update_ok: true,
            },
            MockTeams {
                read: vec![team_a()],
                write: vec![team_a()],
            },
            MockLikes {
                ids: HashSet::new(),
            },
        );
        let perms = UserPermissions::new(&user, &svc.teams);
        let r = svc
            .update_setlist_for_user(
                &perms,
                "id",
                CreateSetlist {
                    title: "t".into(),
                    songs: vec![],
                },
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn blc_setl_002_team_acl_configured() {
        use std::sync::Arc;

        use crate::database::Database;
        use crate::resources::User;
        use crate::test_helpers::{
            configure_personal_team_members, create_user, personal_team_id, test_db,
        };
        use shared::team::TeamRole;

        async fn four_user_setlist_fixture(
        ) -> (Arc<Database>, User, User, User, User, String) {
            let db = test_db().await.expect("db");
            let owner = create_user(&db, "setlist-owner@test.local")
                .await
                .expect("owner");
            let read_u = create_user(&db, "setlist-read@test.local")
                .await
                .expect("read");
            let write_u = create_user(&db, "setlist-write@test.local")
                .await
                .expect("write");
            let noperm = create_user(&db, "setlist-noperm@test.local")
                .await
                .expect("noperm");
            let team_id = personal_team_id(&db, &owner).await.expect("team id");
            configure_personal_team_members(
                &db,
                &owner,
                &team_id,
                vec![
                    (read_u.id.clone(), TeamRole::Guest),
                    (write_u.id.clone(), TeamRole::ContentMaintainer),
                ],
            )
            .await
            .expect("acl");
            (db, owner, read_u, write_u, noperm, team_id)
        }

        let (_db, _owner, _read, _write, _noperm, _team) =
            four_user_setlist_fixture().await;
    }

    #[tokio::test]
    async fn blc_setl_009_create_owner_and_title() {
        use std::sync::Arc;

        use crate::database::Database;
        use crate::error::AppError;
        use crate::resources::User;
        use crate::resources::song::Format;
        use crate::resources::team::UserPermissions;
        use crate::test_helpers::{
            configure_personal_team_members, create_song_with_title, create_user,
            personal_team_id, setlist_service, setlist_with_songs, test_db,
        };
        use shared::api::ListQuery;
        use shared::team::TeamRole;

        async fn four_user_setlist_fixture(
        ) -> (Arc<Database>, User, User, User, User, String) {
            let db = test_db().await.expect("db");
            let owner = create_user(&db, "setl009-owner@test.local")
                .await
                .expect("owner");
            let read_u = create_user(&db, "setl009-read@test.local")
                .await
                .expect("read");
            let write_u = create_user(&db, "setl009-write@test.local")
                .await
                .expect("write");
            let noperm = create_user(&db, "setl009-noperm@test.local")
                .await
                .expect("noperm");
            let team_id = personal_team_id(&db, &owner).await.expect("team id");
            configure_personal_team_members(
                &db,
                &owner,
                &team_id,
                vec![
                    (read_u.id.clone(), TeamRole::Guest),
                    (write_u.id.clone(), TeamRole::ContentMaintainer),
                ],
            )
            .await
            .expect("acl");
            (db, owner, read_u, write_u, noperm, team_id)
        }

        let (db, owner, read_u, write_u, noperm, team_id) =
            four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Setlist Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Setlist Song Two")
            .await
            .expect("s2");

        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let write_p = UserPermissions::new(&write_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs(
                    "Sunday Morning Set",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("create");

        assert_eq!(created.owner, team_id);
        assert_eq!(created.title, "Sunday Morning Set");
        assert_eq!(created.songs.len(), 2);

        let for_delete = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs(
                    "Setlist Delete By Write User",
                    &[(s1.id.as_str(), Some("1"))],
                ),
            )
            .await
            .expect("for_delete");

        let all = sl
            .list_setlists_for_user(&owner_p, ListQuery::default())
            .await
            .expect("list");
        assert_eq!(all.len(), 2);

        let page = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(0).with_page_size(1))
            .await
            .expect("page");
        assert_eq!(page.len(), 1);

        let read_list = sl
            .list_setlists_for_user(&read_p, ListQuery::default())
            .await
            .expect("read list");
        assert_eq!(read_list.len(), 2);

        let noperm_list = sl
            .list_setlists_for_user(&noperm_p, ListQuery::default())
            .await
            .expect("noperm list");
        assert_eq!(noperm_list.len(), 0);

        let q = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_q("Sunday"))
            .await
            .expect("q");
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].id, created.id);

        let q_page = sl
            .list_setlists_for_user(
                &owner_p,
                ListQuery::new()
                    .with_q("Sunday")
                    .with_page(0)
                    .with_page_size(1),
            )
            .await
            .expect("q page");
        assert_eq!(q_page.len(), 1);

        let q_empty = sl
            .list_setlists_for_user(
                &owner_p,
                ListQuery::new().with_q("SetlistNoSuchTokenEver999zz"),
            )
            .await
            .expect("q empty");
        assert_eq!(q_empty.len(), 0);

        let q_blank = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_q(" "))
            .await
            .expect("q blank");
        assert_eq!(q_blank.len(), 2);

        let zero_size = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(0).with_page_size(0))
            .await
            .expect("zero size");
        assert_eq!(zero_size.len(), 2);

        let page_only = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(1))
            .await
            .expect("page only");
        assert_eq!(page_only.len(), 2);

        let page_size_only = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page_size(1))
            .await
            .expect("page only");
        assert_eq!(page_size_only.len(), 2);

        let beyond = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(10).with_page_size(10))
            .await
            .expect("beyond");
        assert_eq!(beyond.len(), 0);

        let g1 = sl
            .get_setlist_for_user(&owner_p, &created.id)
            .await
            .expect("get owner");
        assert_eq!(g1.songs.len(), 2);

        sl.get_setlist_for_user(&read_p, &created.id)
            .await
            .expect("get read");

        let miss = sl.get_setlist_for_user(&noperm_p, &created.id).await;
        assert!(matches!(miss, Err(AppError::NotFound(_))));

        let bad_id = sl.get_setlist_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(bad_id, Err(AppError::InvalidRequest(_))));

        let notfound = sl
            .get_setlist_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(notfound, Err(AppError::NotFound(_))));

        let songs = sl
            .setlist_songs_for_user(&owner_p, &created.id)
            .await
            .expect("songs owner");
        assert_eq!(songs.len(), 2);

        let songs_read = sl
            .setlist_songs_for_user(&read_p, &created.id)
            .await
            .expect("songs read");
        assert_eq!(songs_read.len(), 2);

        let songs_noperm = sl.setlist_songs_for_user(&noperm_p, &created.id).await;
        assert!(matches!(songs_noperm, Err(AppError::NotFound(_))));

        let songs_bad = sl.setlist_songs_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(songs_bad, Err(AppError::InvalidRequest(_))));

        let songs_nf = sl
            .setlist_songs_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(songs_nf, Err(AppError::NotFound(_))));

        let player = sl
            .setlist_player_for_user(&owner_p, &created.id)
            .await
            .expect("player");
        assert_eq!(player.toc().len(), 2);

        sl.setlist_player_for_user(&read_p, &created.id)
            .await
            .expect("player read");

        let pl_noperm = sl.setlist_player_for_user(&noperm_p, &created.id).await;
        assert!(matches!(pl_noperm, Err(AppError::NotFound(_))));

        let pl_bad = sl.setlist_player_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(pl_bad, Err(AppError::InvalidRequest(_))));

        let pl_nf = sl
            .setlist_player_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(pl_nf, Err(AppError::NotFound(_))));

        sl.export_setlist_for_user(&owner_p, &created.id, Format::WorshipPro)
            .await
            .expect("export owner");
        sl.export_setlist_for_user(&read_p, &created.id, Format::WorshipPro)
            .await
            .expect("export read");

        let ex_noperm = sl
            .export_setlist_for_user(&noperm_p, &created.id, Format::WorshipPro)
            .await;
        assert!(matches!(ex_noperm, Err(AppError::NotFound(_))));

        let ex_bad = sl
            .export_setlist_for_user(&owner_p, "song:invalid", Format::WorshipPro)
            .await;
        assert!(matches!(ex_bad, Err(AppError::InvalidRequest(_))));

        let ex_nf = sl
            .export_setlist_for_user(&owner_p, "never-created-setlist", Format::WorshipPro)
            .await;
        assert!(matches!(ex_nf, Err(AppError::NotFound(_))));

        let updated = sl
            .update_setlist_for_user(
                &owner_p,
                &created.id,
                setlist_with_songs(
                    "Updated Sunday Set",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("put owner");
        assert_eq!(updated.title, "Updated Sunday Set");

        let write_updated = sl
            .update_setlist_for_user(
                &write_p,
                &created.id,
                setlist_with_songs(
                    "Write User Updated Set",
                    &[(s1.id.as_str(), Some("10")), (s2.id.as_str(), Some("20"))],
                ),
            )
            .await
            .expect("put write");
        assert_eq!(write_updated.title, "Write User Updated Set");

        let got = sl
            .get_setlist_for_user(&owner_p, &created.id)
            .await
            .expect("get after write");
        assert_eq!(got.title, "Write User Updated Set");

        let put_noperm = sl
            .update_setlist_for_user(
                &noperm_p,
                &created.id,
                setlist_with_songs("Should Fail", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_noperm, Err(AppError::NotFound(_))));

        let put_read = sl
            .update_setlist_for_user(
                &read_p,
                &created.id,
                setlist_with_songs(
                    "Read User Put",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await;
        assert!(matches!(put_read, Err(AppError::NotFound(_))));

        let put_bad = sl
            .update_setlist_for_user(
                &owner_p,
                "song:invalid",
                setlist_with_songs("x", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_bad, Err(AppError::InvalidRequest(_))));

        let put_nf = sl
            .update_setlist_for_user(
                &owner_p,
                "never-created-setlist",
                setlist_with_songs("Unknown Setlist", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_nf, Err(AppError::NotFound(_))));

        let del_noperm = sl.delete_setlist_for_user(&noperm_p, &created.id).await;
        assert!(matches!(del_noperm, Err(AppError::NotFound(_))));

        let del_bad = sl.delete_setlist_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(del_bad, Err(AppError::InvalidRequest(_))));

        sl.delete_setlist_for_user(&write_p, &for_delete.id)
            .await
            .expect("delete write");

        sl.delete_setlist_for_user(&owner_p, &created.id)
            .await
            .expect("delete owner");

        let again = sl.delete_setlist_for_user(&owner_p, &created.id).await;
        assert!(matches!(again, Err(AppError::NotFound(_))));
    }
}
