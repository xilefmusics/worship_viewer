use std::sync::Arc;

use shared::api::ListQuery;
use shared::player::Player;
use shared::setlist::{CreateSetlist, PatchSetlist, Setlist};
use shared::song::Song;

use crate::error::AppError;
use crate::resources::song::LikedSongIds;
use crate::resources::team::{TeamResolver, UserPermissions};

use super::repository::SetlistRepository;
use crate::resources::common::player_from_song_links;

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
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
        let links = self.repo.get_setlist_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn setlist_songs_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
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

    pub async fn patch_setlist_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        patch: PatchSetlist,
    ) -> Result<Setlist, AppError> {
        let current = self.get_setlist_for_user(perms, id).await?;
        let merged = CreateSetlist {
            title: patch.title.unwrap_or(current.title),
            songs: patch.songs.unwrap_or(current.songs),
        };
        self.update_setlist_for_user(perms, id, merged).await
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
    Arc<crate::database::Database>,
>;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use async_trait::async_trait;
    use surrealdb::sql::Thing;

    use shared::api::ListQuery;
    use shared::setlist::{CreateSetlist, Setlist};
    use shared::song::LinkOwned as SongLinkOwned;
    use shared::team::TeamRole;

    use crate::database::Database;
    use crate::error::AppError;
    use crate::resources::User;
    use crate::resources::song::LikedSongIds;
    use crate::resources::team::{TeamResolver, UserPermissions};
    use crate::test_helpers::{
        configure_personal_team_members, create_song_with_title, create_user, personal_team_id,
        setlist_service, setlist_with_songs, test_db,
    };

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
            _read_teams: &[Thing],
            _pagination: ListQuery,
        ) -> Result<Vec<Setlist>, AppError> {
            Ok(self.setlists.clone())
        }

        async fn get_setlist(&self, _read_teams: &[Thing], _id: &str) -> Result<Setlist, AppError> {
            self.get_returns
                .clone()
                .ok_or_else(|| AppError::NotFound("setlist not found".into()))
        }

        async fn get_setlist_songs(
            &self,
            _read_teams: &[Thing],
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
            _write_teams: &[Thing],
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
            _write_teams: &[Thing],
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

    /// Shared integration fixture: owner, reader (Guest), writer (ContentMaintainer), and a
    /// noperm user, all on a fresh isolated in-memory DB. ACL is configured on the owner's
    /// personal team so that read_u can read and write_u can write owner's content.
    async fn four_user_setlist_fixture() -> (Arc<Database>, User, User, User, User, String) {
        let db = test_db().await.expect("db");
        let owner = create_user(&db, "setl-owner@test.local")
            .await
            .expect("owner");
        let read_u = create_user(&db, "setl-read@test.local")
            .await
            .expect("read");
        let write_u = create_user(&db, "setl-write@test.local")
            .await
            .expect("write");
        let noperm = create_user(&db, "setl-noperm@test.local")
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

    /// BLC-SETL-002: team ACL is correctly configured — owner, reader, writer, noperm users
    /// can be set up without error.
    #[tokio::test]
    async fn blc_setl_002_team_acl_configured() {
        let (_db, _owner, _read, _write, _noperm, _team) = four_user_setlist_fixture().await;
    }

    /// BLC-SETL-009a: create sets owner to the owner's personal team and stores title/songs.
    #[tokio::test]
    async fn blc_setl_create_owner_and_title() {
        let (db, owner, _read_u, _write_u, _noperm, team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");
        let owner_p = UserPermissions::new(&owner, &sl.teams);

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
    }

    /// BLC-SETL-009b: list returns correct counts for owner, reader, and noperm user;
    /// pagination by page+page_size works correctly.
    #[tokio::test]
    async fn blc_setl_list_and_pagination() {
        let (db, owner, read_u, _write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        sl.create_setlist_for_user(
            &owner_p,
            setlist_with_songs("Set A", &[(s1.id.as_str(), Some("1"))]),
        )
        .await
        .expect("create A");
        sl.create_setlist_for_user(
            &owner_p,
            setlist_with_songs("Set B", &[(s1.id.as_str(), Some("2"))]),
        )
        .await
        .expect("create B");

        let all = sl
            .list_setlists_for_user(&owner_p, ListQuery::default())
            .await
            .expect("list all");
        assert_eq!(all.len(), 2);

        let page = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(0).with_page_size(1))
            .await
            .expect("page 0 size 1");
        assert_eq!(page.len(), 1);

        let beyond = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(10).with_page_size(10))
            .await
            .expect("beyond");
        assert_eq!(beyond.len(), 0);

        let read_list = sl
            .list_setlists_for_user(&read_p, ListQuery::default())
            .await
            .expect("reader list");
        assert_eq!(read_list.len(), 2);

        let noperm_list = sl
            .list_setlists_for_user(&noperm_p, ListQuery::default())
            .await
            .expect("noperm list");
        assert_eq!(noperm_list.len(), 0);
    }

    /// BLC-SETL-009c: partial pagination parameters (only page or only page_size) and
    /// zero page_size fall back to returning all results.
    #[tokio::test]
    async fn blc_setl_list_partial_pagination() {
        let (db, owner, _read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song")
            .await
            .expect("s");
        let owner_p = UserPermissions::new(&owner, &sl.teams);

        sl.create_setlist_for_user(&owner_p, setlist_with_songs("A", &[(s1.id.as_str(), None)]))
            .await
            .expect("A");
        sl.create_setlist_for_user(&owner_p, setlist_with_songs("B", &[(s1.id.as_str(), None)]))
            .await
            .expect("B");

        let page_only = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(1))
            .await
            .expect("page only");
        assert_eq!(page_only.len(), 2, "page without page_size returns all");

        let page_size_only = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page_size(1))
            .await
            .expect("page_size only");
        assert_eq!(
            page_size_only.len(),
            2,
            "page_size without page returns all"
        );

        let zero_size = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_page(0).with_page_size(0))
            .await
            .expect("zero size");
        assert_eq!(zero_size.len(), 2, "page_size=0 returns all");
    }

    /// BLC-SETL-009d: full-text search with `q` narrows results; blank/whitespace-only q
    /// is treated as no filter; unmatched token returns empty list.
    #[tokio::test]
    async fn blc_setl_search() {
        let (db, owner, _read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song")
            .await
            .expect("s");
        let owner_p = UserPermissions::new(&owner, &sl.teams);

        let sunday = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs("Sunday Morning Set", &[(s1.id.as_str(), None)]),
            )
            .await
            .expect("sunday");
        sl.create_setlist_for_user(
            &owner_p,
            setlist_with_songs("Wednesday Evening Set", &[(s1.id.as_str(), None)]),
        )
        .await
        .expect("wednesday");

        let q = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_q("Sunday"))
            .await
            .expect("q Sunday");
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].id, sunday.id);

        let q_page = sl
            .list_setlists_for_user(
                &owner_p,
                ListQuery::new()
                    .with_q("Sunday")
                    .with_page(0)
                    .with_page_size(1),
            )
            .await
            .expect("q+page");
        assert_eq!(q_page.len(), 1);

        let q_empty = sl
            .list_setlists_for_user(
                &owner_p,
                ListQuery::new().with_q("SetlistNoSuchTokenEver999zz"),
            )
            .await
            .expect("q no match");
        assert_eq!(q_empty.len(), 0);

        let q_blank = sl
            .list_setlists_for_user(&owner_p, ListQuery::new().with_q(" "))
            .await
            .expect("q blank");
        assert_eq!(q_blank.len(), 2);
    }

    /// BLC-SETL-009e: get returns the correct setlist for owner and reader; returns
    /// NotFound for noperm user, InvalidRequest for wrong-table id, NotFound for
    /// non-existent id.
    #[tokio::test]
    async fn blc_setl_get_acl() {
        let (db, owner, read_u, _write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs(
                    "Sunday Set",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("create");

        let g = sl
            .get_setlist_for_user(&owner_p, &created.id)
            .await
            .expect("get owner");
        assert_eq!(g.songs.len(), 2);

        sl.get_setlist_for_user(&read_p, &created.id)
            .await
            .expect("get reader");

        let miss = sl.get_setlist_for_user(&noperm_p, &created.id).await;
        assert!(matches!(miss, Err(AppError::NotFound(_))));

        let bad_id = sl.get_setlist_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(bad_id, Err(AppError::InvalidRequest(_))));

        let notfound = sl
            .get_setlist_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(notfound, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-009f: setlist_songs returns songs for owner and reader; NotFound for
    /// noperm, InvalidRequest for wrong-table id, NotFound for non-existent id.
    #[tokio::test]
    async fn blc_setl_songs_acl() {
        let (db, owner, read_u, _write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs(
                    "Sunday Set",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("create");

        let songs = sl
            .setlist_songs_for_user(&owner_p, &created.id)
            .await
            .expect("songs owner");
        assert_eq!(songs.len(), 2);

        let songs_read = sl
            .setlist_songs_for_user(&read_p, &created.id)
            .await
            .expect("songs reader");
        assert_eq!(songs_read.len(), 2);

        let songs_noperm = sl.setlist_songs_for_user(&noperm_p, &created.id).await;
        assert!(matches!(songs_noperm, Err(AppError::NotFound(_))));

        let songs_bad = sl.setlist_songs_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(songs_bad, Err(AppError::InvalidRequest(_))));

        let songs_nf = sl
            .setlist_songs_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(songs_nf, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-009g: player returns a toc for owner and reader; NotFound for noperm,
    /// InvalidRequest for wrong-table id, NotFound for non-existent id.
    #[tokio::test]
    async fn blc_setl_player_acl() {
        let (db, owner, read_u, _write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs(
                    "Sunday Set",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("create");

        let player = sl
            .setlist_player_for_user(&owner_p, &created.id)
            .await
            .expect("player owner");
        assert_eq!(player.toc().len(), 2);

        sl.setlist_player_for_user(&read_p, &created.id)
            .await
            .expect("player reader");

        let pl_noperm = sl.setlist_player_for_user(&noperm_p, &created.id).await;
        assert!(matches!(pl_noperm, Err(AppError::NotFound(_))));

        let pl_bad = sl.setlist_player_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(pl_bad, Err(AppError::InvalidRequest(_))));

        let pl_nf = sl
            .setlist_player_for_user(&owner_p, "never-created-setlist")
            .await;
        assert!(matches!(pl_nf, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-009i: update succeeds for owner (title changes) and for write user;
    /// writer's change is visible to owner on subsequent get; read user and noperm user
    /// are rejected; wrong-table id returns InvalidRequest; non-existent id returns
    /// NotFound.
    #[tokio::test]
    async fn blc_setl_update_acl() {
        let (db, owner, read_u, write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
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
                    "Original Title",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("create");

        let updated = sl
            .update_setlist_for_user(
                &owner_p,
                &created.id,
                setlist_with_songs(
                    "Owner Updated Title",
                    &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
                ),
            )
            .await
            .expect("update owner");
        assert_eq!(updated.title, "Owner Updated Title");

        let write_updated = sl
            .update_setlist_for_user(
                &write_p,
                &created.id,
                setlist_with_songs(
                    "Write User Updated Title",
                    &[(s1.id.as_str(), Some("10")), (s2.id.as_str(), Some("20"))],
                ),
            )
            .await
            .expect("update write user");
        assert_eq!(write_updated.title, "Write User Updated Title");

        let after_write = sl
            .get_setlist_for_user(&owner_p, &created.id)
            .await
            .expect("get after write");
        assert_eq!(after_write.title, "Write User Updated Title");

        let put_read = sl
            .update_setlist_for_user(
                &read_p,
                &created.id,
                setlist_with_songs("Read User Put", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_read, Err(AppError::NotFound(_))));

        let put_noperm = sl
            .update_setlist_for_user(
                &noperm_p,
                &created.id,
                setlist_with_songs("Should Fail", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_noperm, Err(AppError::NotFound(_))));

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
                setlist_with_songs("Unknown", &[(s1.id.as_str(), None)]),
            )
            .await;
        assert!(matches!(put_nf, Err(AppError::NotFound(_))));
    }

    /// PATCH-SETL-001: patch with only title changes title, songs remain unchanged.
    #[tokio::test]
    async fn patch_setlist_title_only_leaves_songs_unchanged() {
        use shared::setlist::PatchSetlist;
        let (db, owner, _read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let owner_p = UserPermissions::new(&owner, &sl.teams);

        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs("Original Title", &[(s1.id.as_str(), Some("1"))]),
            )
            .await
            .expect("create");

        let patched = sl
            .patch_setlist_for_user(
                &owner_p,
                &created.id,
                PatchSetlist {
                    title: Some("New Title".into()),
                    songs: None,
                },
            )
            .await
            .expect("patch");

        assert_eq!(patched.title, "New Title");
        assert_eq!(
            patched.songs.len(),
            created.songs.len(),
            "songs must be unchanged"
        );
    }

    /// PATCH-SETL-002: PATCH on non-existent setlist returns NotFound.
    #[tokio::test]
    async fn patch_setlist_not_found() {
        use shared::setlist::PatchSetlist;
        let (db, owner, _read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let r = sl
            .patch_setlist_for_user(
                &owner_p,
                "never-existed-setlist",
                PatchSetlist {
                    title: Some("x".into()),
                    songs: None,
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// PATCH-SETL-003: read-only guest cannot PATCH a setlist.
    #[tokio::test]
    async fn patch_setlist_guest_cannot_patch() {
        use shared::setlist::PatchSetlist;
        let (db, owner, read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song")
            .await
            .expect("s");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let read_p = UserPermissions::new(&read_u, &sl.teams);
        let created = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs("Title", &[(s1.id.as_str(), None)]),
            )
            .await
            .expect("create");
        let r = sl
            .patch_setlist_for_user(
                &read_p,
                &created.id,
                PatchSetlist {
                    title: Some("Hacked".into()),
                    songs: None,
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn patch_setlist_all_field_combinations() {
        use shared::setlist::PatchSetlist;

        let (db, owner, _read_u, _write_u, _noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");
        let owner_p = UserPermissions::new(&owner, &sl.teams);

        for mask in 0u8..4 {
            let created = sl
                .create_setlist_for_user(
                    &owner_p,
                    setlist_with_songs("BaseTitle", &[(s1.id.as_str(), Some("1"))]),
                )
                .await
                .expect("create");

            let include_title = (mask & 0b01) != 0;
            let include_songs = (mask & 0b10) != 0;

            let patched = sl
                .patch_setlist_for_user(
                    &owner_p,
                    &created.id,
                    PatchSetlist {
                        title: include_title.then_some("PatchedTitle".into()),
                        songs: include_songs.then_some(vec![shared::song::Link {
                            id: s2.id.clone(),
                            nr: Some("9".into()),
                            key: None,
                        }]),
                    },
                )
                .await
                .expect("patch");

            let expected_title = if include_title {
                "PatchedTitle"
            } else {
                "BaseTitle"
            };
            assert_eq!(
                patched.title, expected_title,
                "mask={mask:02b}: title mismatch"
            );
            if include_songs {
                assert_eq!(patched.songs.len(), 1, "mask={mask:02b}: expected 1 song");
                assert_eq!(
                    patched.songs[0].id, s2.id,
                    "mask={mask:02b}: songs replacement mismatch"
                );
            } else {
                assert_eq!(
                    patched.songs[0].id, s1.id,
                    "mask={mask:02b}: songs should remain unchanged"
                );
            }
        }
    }

    /// BLC-SETL-009j: delete is rejected for noperm and wrong-table id; write user can
    /// delete; owner can delete; double-delete returns NotFound.
    #[tokio::test]
    async fn blc_setl_delete_acl() {
        let (db, owner, _read_u, write_u, noperm, _team_id) = four_user_setlist_fixture().await;
        let sl = setlist_service(&db);
        let s1 = create_song_with_title(&db, &owner, "Song")
            .await
            .expect("s");
        let owner_p = UserPermissions::new(&owner, &sl.teams);
        let write_p = UserPermissions::new(&write_u, &sl.teams);
        let noperm_p = UserPermissions::new(&noperm, &sl.teams);

        let owner_setlist = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs("Owner's Setlist", &[(s1.id.as_str(), Some("1"))]),
            )
            .await
            .expect("create owner setlist");

        let write_setlist = sl
            .create_setlist_for_user(
                &owner_p,
                setlist_with_songs("Write Setlist", &[(s1.id.as_str(), Some("1"))]),
            )
            .await
            .expect("create write setlist");

        let del_noperm = sl
            .delete_setlist_for_user(&noperm_p, &owner_setlist.id)
            .await;
        assert!(matches!(del_noperm, Err(AppError::NotFound(_))));

        let del_bad = sl.delete_setlist_for_user(&owner_p, "song:invalid").await;
        assert!(matches!(del_bad, Err(AppError::InvalidRequest(_))));

        sl.delete_setlist_for_user(&write_p, &write_setlist.id)
            .await
            .expect("write user delete");

        sl.delete_setlist_for_user(&owner_p, &owner_setlist.id)
            .await
            .expect("owner delete");

        let again = sl
            .delete_setlist_for_user(&owner_p, &owner_setlist.id)
            .await;
        assert!(matches!(again, Err(AppError::NotFound(_))));
    }
}
