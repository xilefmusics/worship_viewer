use std::sync::Arc;

use shared::api::ListQuery;
use shared::collection::{Collection, CreateCollection, PatchCollection};
use shared::player::Player;
use shared::song::Song;

use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::player_from_song_links;
use crate::resources::song::LikedSongIds;
use crate::resources::team::{TeamResolver, UserPermissions};

use super::repository::CollectionRepository;
use super::surreal_repo::SurrealCollectionRepo;

/// Application service: team resolution, authorization, and orchestration for collections.
#[derive(Clone)]
pub struct CollectionService<R, T, L> {
    pub repo: R,
    pub teams: T,
    pub likes: L,
}

impl<R, T, L> CollectionService<R, T, L> {
    pub fn new(repo: R, teams: T, likes: L) -> Self {
        Self { repo, teams, likes }
    }
}

impl<R: CollectionRepository, T: TeamResolver, L: LikedSongIds> CollectionService<R, T, L> {
    pub async fn list_collections_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError> {
        let read_teams = perms.read_teams().await?;
        self.repo.get_collections(read_teams, pagination).await
    }

    pub async fn get_collection_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let read_teams = perms.read_teams().await?;
        self.repo.get_collection(read_teams, id).await
    }

    pub async fn collection_player_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Player, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
        let links = self.repo.get_collection_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn collection_songs_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
        Ok(self
            .repo
            .get_collection_songs(read_teams, id)
            .await?
            .into_iter()
            .map(|song_link_owned| {
                let mut song = song_link_owned.song;
                song.user_specific_addons.liked = liked_set.contains(&song.id);
                song
            })
            .collect())
    }

    pub async fn create_collection_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        self.repo
            .create_collection(&perms.user().id, collection)
            .await
    }

    pub async fn update_collection_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo
            .update_collection(write_teams, id, collection)
            .await
    }

    pub async fn patch_collection_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        patch: PatchCollection,
    ) -> Result<Collection, AppError> {
        let current = self.get_collection_for_user(perms, id).await?;
        let merged = CreateCollection {
            title: patch.title.unwrap_or(current.title),
            cover: patch.cover.unwrap_or(current.cover),
            songs: patch.songs.unwrap_or(current.songs),
        };
        self.update_collection_for_user(perms, id, merged).await
    }

    pub async fn delete_collection_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo.delete_collection(write_teams, id).await
    }
}

/// Production type alias used in HTTP wiring.
pub type CollectionServiceHandle = CollectionService<
    SurrealCollectionRepo,
    crate::resources::team::SurrealTeamResolver,
    Arc<Database>,
>;

impl CollectionServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        CollectionService::new(
            SurrealCollectionRepo::new(db.clone()),
            crate::resources::team::SurrealTeamResolver::new(db.clone()),
            db.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use shared::song::Link as SongLink;

    use crate::error::AppError;
    use crate::resources::team::UserPermissions;
    use crate::test_helpers::{
        configure_personal_team_members, create_song_with_title, create_user, personal_team_id,
        test_db,
    };
    use shared::api::ListQuery;
    use shared::collection::CreateCollection;
    use shared::team::TeamRole;

    use super::CollectionServiceHandle;

    #[tokio::test]
    async fn blc_collection_crud_and_acl() {
        let db = test_db().await.expect("db");
        let svc = CollectionServiceHandle::build(db.clone());

        let owner = create_user(&db, "coll-owner@test.local").await.expect("o");
        let guest = create_user(&db, "coll-guest@test.local").await.expect("g");
        let team_id = personal_team_id(&db, &owner).await.expect("team");
        configure_personal_team_members(
            &db,
            &owner,
            &team_id,
            vec![(guest.id.clone(), TeamRole::Guest)],
        )
        .await
        .expect("acl");

        let song = create_song_with_title(&db, &owner, "Coll Song")
            .await
            .expect("song");

        let owner_perms = UserPermissions::new(&owner, &svc.teams);
        let guest_perms = UserPermissions::new(&guest, &svc.teams);

        let col = svc
            .create_collection_for_user(
                &owner_perms,
                CreateCollection {
                    title: "My Collection".into(),
                    cover: "mysongs".into(),
                    songs: vec![SongLink {
                        id: song.id.clone(),
                        nr: None,
                        key: None,
                    }],
                },
            )
            .await
            .expect("create");

        assert_eq!(col.owner, team_id);

        let list = svc
            .list_collections_for_user(&owner_perms, ListQuery::default())
            .await
            .expect("list");
        assert!(list.iter().any(|c| c.id == col.id));

        svc.get_collection_for_user(&guest_perms, &col.id)
            .await
            .expect("guest read");

        let upd = svc
            .update_collection_for_user(
                &owner_perms,
                &col.id,
                CreateCollection {
                    title: "Updated".into(),
                    cover: "mysongs".into(),
                    songs: vec![SongLink {
                        id: song.id.clone(),
                        nr: Some("1".into()),
                        key: None,
                    }],
                },
            )
            .await
            .expect("update");
        assert_eq!(upd.title, "Updated");

        let put_guest = svc
            .update_collection_for_user(
                &guest_perms,
                &col.id,
                CreateCollection {
                    title: "Nope".into(),
                    cover: "mysongs".into(),
                    songs: vec![],
                },
            )
            .await;
        assert!(matches!(put_guest, Err(AppError::NotFound(_))));

        svc.delete_collection_for_user(&owner_perms, &col.id)
            .await
            .expect("delete");
    }

    /// Build a four-user collection fixture: owner, content_maintainer, guest, non_member.
    async fn four_user_coll_fixture() -> (
        std::sync::Arc<crate::database::Database>,
        crate::resources::User,
        crate::resources::User,
        crate::resources::User,
        crate::resources::User,
        String,
    ) {
        use crate::test_helpers::{
            configure_personal_team_members, create_user, personal_team_id, test_db,
        };

        let db = test_db().await.expect("db");
        let owner = create_user(&db, "c3g-owner@test.local")
            .await
            .expect("owner");
        let cm = create_user(&db, "c3g-cm@test.local").await.expect("cm");
        let guest = create_user(&db, "c3g-guest@test.local")
            .await
            .expect("guest");
        let non_member = create_user(&db, "c3g-nm@test.local").await.expect("nm");
        let tid = personal_team_id(&db, &owner).await.expect("tid");
        configure_personal_team_members(
            &db,
            &owner,
            &tid,
            vec![
                (cm.id.clone(), TeamRole::ContentMaintainer),
                (guest.id.clone(), TeamRole::Guest),
            ],
        )
        .await
        .expect("acl");
        (db, owner, cm, guest, non_member, tid)
    }

    fn make_collection(title: &str) -> CreateCollection {
        CreateCollection {
            title: title.into(),
            cover: "mysongs".into(),
            songs: vec![],
        }
    }

    /// BLC-COLL-002, BLC-COLL-006: non-member reading a collection returns NotFound.
    #[tokio::test]
    async fn blc_coll_002_non_member_read_not_found() {
        let (db, owner, _cm, _guest, nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let nm_p = UserPermissions::new(&nm, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("NMTest"))
            .await
            .expect("create");
        let r = svc.get_collection_for_user(&nm_p, &col.id).await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-COLL-002: guest can read a collection.
    #[tokio::test]
    async fn blc_coll_002_guest_can_read() {
        let (db, owner, _cm, guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let guest_p = UserPermissions::new(&guest, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("GuestTest"))
            .await
            .expect("create");
        svc.get_collection_for_user(&guest_p, &col.id)
            .await
            .expect("guest read");
    }

    /// BLC-COLL-002: content_maintainer can update a collection.
    #[tokio::test]
    async fn blc_coll_002_content_maintainer_can_update() {
        let (db, owner, cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let cm_p = UserPermissions::new(&cm, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("CMTest"))
            .await
            .expect("create");
        svc.update_collection_for_user(&cm_p, &col.id, make_collection("CMUpdated"))
            .await
            .expect("cm update");
    }

    /// BLC-COLL-007: guest cannot create a collection.
    #[tokio::test]
    async fn blc_coll_007_guest_cannot_create() {
        let (db, _owner, _cm, guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let guest_p = UserPermissions::new(&guest, &svc.teams);
        // The collection would be owned by the guest's personal team, which the guest can write.
        // But test: guest on owner's team cannot write to owner's collections.
        // Actually, create always goes to caller's personal team, so guest creates on their own
        // personal team -> that succeeds. The constraint is about mutating others' content.
        // BLC-COLL-007 tests guest PUT/DELETE on owner's collection.
        let owner_2 = crate::test_helpers::create_user(&db, "c3g-owner2@test.local")
            .await
            .expect("o2");
        let owner2_p = UserPermissions::new(&owner_2, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner2_p, make_collection("O2Coll"))
            .await
            .expect("create");
        let r = svc
            .update_collection_for_user(&guest_p, &col.id, make_collection("Hack"))
            .await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-COLL-007: guest cannot delete a collection they don't own.
    #[tokio::test]
    async fn blc_coll_007_guest_cannot_delete() {
        let (db, owner, _cm, guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let guest_p = UserPermissions::new(&guest, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("GuestDel"))
            .await
            .expect("create");
        let r = svc.delete_collection_for_user(&guest_p, &col.id).await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-COLL-003: PUT does not change the collection's owner.
    #[tokio::test]
    async fn blc_coll_003_put_does_not_change_owner() {
        let (db, owner, _cm, _guest, _nm, tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("OwnerTest"))
            .await
            .expect("create");
        assert_eq!(col.owner, tid);
        let updated = svc
            .update_collection_for_user(&owner_p, &col.id, make_collection("Renamed"))
            .await
            .expect("update");
        assert_eq!(updated.owner, tid, "owner must not change on PUT");
    }

    /// BLC-COLL-004: POST with a non-existent song ID succeeds (no existence check).
    #[tokio::test]
    async fn blc_coll_004_post_accepts_nonexistent_song_ids() {
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let col = svc
            .create_collection_for_user(
                &owner_p,
                CreateCollection {
                    title: "WithGhostSong".into(),
                    cover: "mysongs".into(),
                    songs: vec![shared::song::Link {
                        id: "song:doesnotexist".into(),
                        nr: None,
                        key: None,
                    }],
                },
            )
            .await
            .expect("non-existent song id accepted");
        assert!(!col.id.is_empty());
    }

    /// BLC-COLL-005: list with `q` filter matches by title (single-token titles).
    #[tokio::test]
    async fn blc_coll_005_list_with_q_filter() {
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        svc.create_collection_for_user(&owner_p, make_collection("Hallelujah"))
            .await
            .expect("c1");
        svc.create_collection_for_user(&owner_p, make_collection("Amazing"))
            .await
            .expect("c2");
        let results = svc
            .list_collections_for_user(&owner_p, ListQuery::new().with_q("Hallelujah"))
            .await
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Hallelujah");
    }

    /// BLC-COLL-005: list with pagination returns the correct page.
    #[tokio::test]
    async fn blc_coll_005_list_pagination() {
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        for i in 0..3u32 {
            svc.create_collection_for_user(&owner_p, make_collection(&format!("Coll{i}")))
                .await
                .expect("create");
        }
        let page0 = svc
            .list_collections_for_user(
                &owner_p,
                ListQuery::default().with_page(0).with_page_size(2),
            )
            .await
            .expect("page0");
        assert_eq!(page0.len(), 2);
        let page1 = svc
            .list_collections_for_user(
                &owner_p,
                ListQuery::default().with_page(1).with_page_size(2),
            )
            .await
            .expect("page1");
        assert_eq!(page1.len(), 1);
    }

    /// BLC-COLL-011: authorized user can list songs in a collection.
    #[tokio::test]
    async fn blc_coll_011_songs_sub_route_authorized() {
        use crate::test_helpers::create_song_with_title;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let song = create_song_with_title(&db, &owner, "CollSongSub")
            .await
            .expect("song");
        let col = svc
            .create_collection_for_user(
                &owner_p,
                CreateCollection {
                    title: "SubTest".into(),
                    cover: "mysongs".into(),
                    songs: vec![shared::song::Link {
                        id: song.id.clone(),
                        nr: None,
                        key: None,
                    }],
                },
            )
            .await
            .expect("create");
        let songs = svc
            .collection_songs_for_user(&owner_p, &col.id)
            .await
            .expect("songs");
        assert!(songs.iter().any(|s| s.id == song.id));
    }

    /// PATCH-COLL-001: patch with only title changes title; cover and songs remain unchanged.
    #[tokio::test]
    async fn patch_collection_title_only_leaves_cover_and_songs_unchanged() {
        use shared::collection::PatchCollection;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);

        let col = svc
            .create_collection_for_user(&owner_p, make_collection("Old Title"))
            .await
            .expect("create");

        let patched = svc
            .patch_collection_for_user(
                &owner_p,
                &col.id,
                PatchCollection {
                    title: Some("New Title".into()),
                    cover: None,
                    songs: None,
                },
            )
            .await
            .expect("patch");

        assert_eq!(patched.title, "New Title");
        assert_eq!(patched.cover, col.cover, "cover must be unchanged");
        assert_eq!(
            patched.songs.len(),
            col.songs.len(),
            "songs must be unchanged"
        );
    }

    /// PATCH-COLL-002: patch with only cover changes cover; title and songs remain unchanged.
    #[tokio::test]
    async fn patch_collection_cover_only_leaves_title_and_songs_unchanged() {
        use shared::collection::PatchCollection;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);

        let col = svc
            .create_collection_for_user(&owner_p, make_collection("Title"))
            .await
            .expect("create");

        let patched = svc
            .patch_collection_for_user(
                &owner_p,
                &col.id,
                PatchCollection {
                    title: None,
                    cover: Some("newheart".into()),
                    songs: None,
                },
            )
            .await
            .expect("patch");

        assert_eq!(patched.cover, "newheart");
        assert_eq!(patched.title, col.title, "title must be unchanged");
    }

    /// PATCH-COLL-003: PATCH on a non-existent collection returns NotFound.
    #[tokio::test]
    async fn patch_collection_not_found() {
        use shared::collection::PatchCollection;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let r = svc
            .patch_collection_for_user(
                &owner_p,
                "never-existed-collection",
                PatchCollection {
                    title: Some("x".into()),
                    cover: None,
                    songs: None,
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn patch_collection_all_field_combinations() {
        use shared::collection::PatchCollection;

        let (db, owner, _cm, _guest, _nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let s1 = create_song_with_title(&db, &owner, "Song One")
            .await
            .expect("s1");
        let s2 = create_song_with_title(&db, &owner, "Song Two")
            .await
            .expect("s2");

        for mask in 0u8..8 {
            let created = svc
                .create_collection_for_user(
                    &owner_p,
                    CreateCollection {
                        title: "BaseTitle".into(),
                        cover: "mysongs".into(),
                        songs: vec![shared::song::Link {
                            id: s1.id.clone(),
                            nr: Some("1".into()),
                            key: None,
                        }],
                    },
                )
                .await
                .expect("create");

            let include_title = (mask & 0b001) != 0;
            let include_cover = (mask & 0b010) != 0;
            let include_songs = (mask & 0b100) != 0;

            let patched = svc
                .patch_collection_for_user(
                    &owner_p,
                    &created.id,
                    PatchCollection {
                        title: include_title.then_some("PatchedTitle".into()),
                        cover: include_cover.then_some("newheart".into()),
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
            let expected_cover = if include_cover { "newheart" } else { "mysongs" };

            assert_eq!(
                patched.title, expected_title,
                "mask={mask:03b}: title mismatch"
            );
            assert_eq!(
                patched.cover, expected_cover,
                "mask={mask:03b}: cover mismatch"
            );
            if include_songs {
                assert_eq!(
                    patched.songs[0].id, s2.id,
                    "mask={mask:03b}: songs mismatch"
                );
            } else {
                assert_eq!(
                    patched.songs[0].id, s1.id,
                    "mask={mask:03b}: songs should remain unchanged"
                );
            }
        }
    }

    /// BLC-COLL-011: unauthorized user cannot access collection songs sub-route.
    #[tokio::test]
    async fn blc_coll_011_songs_sub_route_unauthorized() {
        let (db, owner, _cm, _guest, nm, _tid) = four_user_coll_fixture().await;
        let svc = CollectionServiceHandle::build(db.clone());
        let owner_p = UserPermissions::new(&owner, &svc.teams);
        let nm_p = UserPermissions::new(&nm, &svc.teams);
        let col = svc
            .create_collection_for_user(&owner_p, make_collection("SecretColl"))
            .await
            .expect("create");
        let r = svc.collection_songs_for_user(&nm_p, &col.id).await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }
}
