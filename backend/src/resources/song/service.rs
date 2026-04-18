use std::sync::Arc;

use async_trait::async_trait;

use shared::api::SongListQuery;
use shared::like::LikeStatus;
use shared::patch::Patch;
use shared::player::Player;
use shared::song::{
    CreateSong, Link as SongLink, LinkOwned as SongLinkOwned, PatchSong, PatchSongData, Song,
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::collection::CollectionRepository;

use crate::resources::team::{TeamResolver, UserPermissions};
use crate::resources::user::SurrealUserRepo;
use crate::resources::user::UserRepository;
use shared::collection::CreateCollection;

use super::liked::LikedSongIds;
use super::repository::{SongRepository, SongUpsertOutcome};
use super::surreal_repo::SurrealSongRepo;

/// Abstraction over updating a user's default collection reference.
#[async_trait]
pub trait UserCollectionUpdater: Send + Sync {
    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError>;

    async fn clear_default_collection(&self, user_id: &str) -> Result<(), AppError>;
}

#[async_trait]
impl UserCollectionUpdater for Arc<SurrealUserRepo> {
    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError> {
        (**self)
            .set_default_collection(user_id, collection_id)
            .await
    }

    async fn clear_default_collection(&self, user_id: &str) -> Result<(), AppError> {
        (**self).clear_default_collection(user_id).await
    }
}

/// Application service: team resolution, authorization, and orchestration for songs.
#[derive(Clone)]
pub struct SongService<R, T, L, C, U> {
    pub repo: R,
    pub teams: Arc<T>,
    pub likes: L,
    pub collections: C,
    pub user_updater: U,
}

impl<R, T, L, C, U> SongService<R, T, L, C, U> {
    pub fn new(repo: R, teams: Arc<T>, likes: L, collections: C, user_updater: U) -> Self {
        Self {
            repo,
            teams,
            likes,
            collections,
            user_updater,
        }
    }
}

impl<
    R: SongRepository,
    T: TeamResolver,
    L: LikedSongIds,
    C: CollectionRepository,
    U: UserCollectionUpdater,
> SongService<R, T, L, C, U>
{
    fn merge_song_data(
        mut current: chordlib::types::Song,
        patch: PatchSongData,
    ) -> chordlib::types::Song {
        if let Some(v) = patch.titles {
            current.titles = v;
        }
        match patch.subtitle {
            Patch::Missing => {}
            Patch::Null => current.subtitle = None,
            Patch::Value(v) => current.subtitle = Some(v),
        }
        match patch.copyright {
            Patch::Missing => {}
            Patch::Null => current.copyright = None,
            Patch::Value(v) => current.copyright = Some(v),
        }
        match patch.key {
            Patch::Missing => {}
            Patch::Null => current.key = None,
            Patch::Value(v) => current.key = Some(v),
        }
        if let Some(v) = patch.artists {
            current.artists = v;
        }
        if let Some(v) = patch.languages {
            current.languages = v;
        }
        match patch.tempo {
            Patch::Missing => {}
            Patch::Null => current.tempo = None,
            Patch::Value(v) => current.tempo = Some(v),
        }
        match patch.time {
            Patch::Missing => {}
            Patch::Null => current.time = None,
            Patch::Value(v) => current.time = Some(v),
        }
        if let Some(v) = patch.tags {
            current.tags = v;
        }
        if let Some(v) = patch.sections {
            current.sections = v;
        }
        current
    }

    pub async fn list_songs_for_user(
        &self,
        perms: &UserPermissions<T>,
        query: SongListQuery,
    ) -> Result<Vec<Song>, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
        Ok(self
            .repo
            .get_songs(read_teams, query)
            .await?
            .into_iter()
            .map(|mut song| {
                song.user_specific_addons.liked = liked_set.contains(&song.id);
                song
            })
            .collect())
    }

    pub async fn count_songs_for_user(
        &self,
        perms: &UserPermissions<T>,
        query: &SongListQuery,
    ) -> Result<u64, AppError> {
        let read_teams = perms.read_teams().await?;
        self.repo.count_songs(read_teams, query).await
    }

    pub async fn get_song_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
    ) -> Result<Song, AppError> {
        let user_id = perms.user().id.clone();
        let (liked_set, read_teams) =
            tokio::try_join!(self.likes.liked_song_ids(&user_id), perms.read_teams())?;
        let mut song = self.repo.get_song(read_teams, id).await?;
        song.user_specific_addons.liked = liked_set.contains(&song.id);
        Ok(song)
    }

    pub async fn song_player_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
    ) -> Result<Player, AppError> {
        let read_teams = perms.read_teams().await?;
        Ok(Player::from(SongLinkOwned {
            song: self.repo.get_song(read_teams, id).await?,
            nr: None,
            key: None,
            liked: self
                .repo
                .get_song_like(read_teams, &perms.user().id, id)
                .await?,
        }))
    }

    pub async fn create_song_for_user(
        &self,
        perms: &UserPermissions<T>,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let created = self.repo.create_song(&perms.user().id, song).await?;
        let mut default_id = perms.user().default_collection.clone();

        loop {
            let write_teams = perms.write_teams().await?;
            if let Some(collection_id) = default_id.as_ref() {
                match self
                    .collections
                    .add_song_to_collection(
                        write_teams,
                        collection_id,
                        SongLink {
                            id: created.id.clone(),
                            nr: None,
                            key: None,
                        },
                    )
                    .await
                {
                    Ok(()) => break,
                    Err(e) if matches!(&e, AppError::NotFound(_) | AppError::Conflict(_)) => {
                        self.user_updater
                            .clear_default_collection(&perms.user().id)
                            .await?;
                        default_id = None;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let collection = self
                    .collections
                    .create_collection(
                        &perms.user().id,
                        CreateCollection {
                            title: "Default".to_string(),
                            cover: "mysongs".to_string(),
                            songs: vec![SongLink {
                                id: created.id.clone(),
                                nr: None,
                                key: None,
                            }],
                        },
                    )
                    .await?;
                self.user_updater
                    .set_default_collection(&perms.user().id, &collection.id)
                    .await?;
                break;
            }
        }

        Ok(created)
    }

    pub async fn update_song_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
        song: CreateSong,
    ) -> Result<SongUpsertOutcome, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo
            .update_song(write_teams, &perms.user().id, id, song)
            .await
    }

    pub async fn patch_song_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
        patch: PatchSong,
    ) -> Result<Song, AppError> {
        let current = self.get_song_for_user(perms, id).await?;
        let merged = CreateSong {
            not_a_song: patch.not_a_song.unwrap_or(current.not_a_song),
            blobs: patch.blobs.unwrap_or(current.blobs),
            data: patch
                .data
                .map(|song_data_patch| Self::merge_song_data(current.data.clone(), song_data_patch))
                .unwrap_or(current.data),
        };
        self.update_song_for_user(perms, id, merged)
            .await
            .map(SongUpsertOutcome::into_song)
    }

    pub async fn delete_song_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
    ) -> Result<Song, AppError> {
        let write_teams = perms.write_teams().await?;
        self.repo.delete_song(write_teams, id).await
    }

    pub async fn song_like_status_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = perms.read_teams().await?;
        let liked = self
            .repo
            .get_song_like(read_teams, &perms.user().id, id)
            .await?;
        Ok(LikeStatus { liked })
    }

    pub async fn set_song_like_status_for_user(
        &self,
        perms: &UserPermissions<T>,
        id: &str,
        liked: bool,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = perms.read_teams().await?;
        let liked = self
            .repo
            .set_song_like(read_teams, &perms.user().id, id, liked)
            .await?;
        Ok(LikeStatus { liked })
    }
}

/// Production type alias used in HTTP wiring.
pub type SongServiceHandle = SongService<
    SurrealSongRepo,
    crate::resources::team::SurrealTeamResolver,
    Arc<Database>,
    crate::resources::collection::SurrealCollectionRepo,
    Arc<SurrealUserRepo>,
>;

impl SongServiceHandle {
    pub fn build(db: Arc<Database>) -> Self {
        Self::build_with_team_resolver(
            db.clone(),
            Arc::new(crate::resources::team::SurrealTeamResolver::new(db.clone())),
        )
    }

    pub fn build_with_team_resolver(
        db: Arc<Database>,
        teams: Arc<crate::resources::team::SurrealTeamResolver>,
    ) -> Self {
        SongService::new(
            SurrealSongRepo::new(db.clone()),
            teams,
            db.clone(),
            crate::resources::collection::SurrealCollectionRepo::new(db.clone()),
            Arc::new(SurrealUserRepo::new(db.clone())),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::team::UserPermissions;
    use crate::test_helpers::{
        configure_personal_team_members, create_song_with_title, create_user, personal_team_id,
        setlist_service, setlist_with_songs, test_db,
    };
    use shared::api::ListQuery;
    use shared::team::TeamRole;

    use super::SongServiceHandle;

    #[tokio::test]
    async fn blc_song_crud_search_likes() {
        let db = test_db().await.expect("db");
        let svc = SongServiceHandle::build(db.clone());

        let owner = create_user(&db, "song-owner@test.local").await.expect("o");
        let other = create_user(&db, "song-other@test.local").await.expect("x");
        let team_id = personal_team_id(&db, &owner).await.expect("team");
        configure_personal_team_members(
            &db,
            &owner,
            &team_id,
            vec![(other.id.clone(), TeamRole::Guest)],
        )
        .await
        .expect("acl");

        let s1 = create_song_with_title(&db, &owner, "Unique Song Alpha")
            .await
            .expect("s1");
        let _s2 = create_song_with_title(&db, &owner, "Other Beta")
            .await
            .expect("s2");

        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let other_p = UserPermissions::from_ref(&other, &svc.teams);

        let list = svc
            .list_songs_for_user(&owner_p, ListQuery::default().into())
            .await
            .expect("list");
        assert!(list.len() >= 2);

        let q = svc
            .list_songs_for_user(&owner_p, ListQuery::new().with_q("Alpha").into())
            .await
            .expect("search");
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].id, s1.id);

        svc.get_song_for_user(&owner_p, &s1.id).await.expect("get");
        svc.get_song_for_user(&other_p, &s1.id)
            .await
            .expect("guest read");

        let bad = svc.get_song_for_user(&owner_p, "setlist:not-a-song").await;
        assert!(bad.is_err(), "wrong table id should not resolve: {bad:?}");

        svc.set_song_like_status_for_user(&owner_p, &s1.id, true)
            .await
            .expect("like");
        let st = svc
            .song_like_status_for_user(&owner_p, &s1.id)
            .await
            .expect("like status");
        assert!(st.liked);

        svc.delete_song_for_user(&owner_p, &s1.id)
            .await
            .expect("del");
    }

    /// Build a four-user song fixture: owner, content_maintainer, guest, non_member.
    async fn four_user_song_fixture() -> (
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
        let owner = create_user(&db, "s3h-owner@test.local")
            .await
            .expect("owner");
        let cm = create_user(&db, "s3h-cm@test.local").await.expect("cm");
        let guest_u = create_user(&db, "s3h-guest@test.local")
            .await
            .expect("guest");
        let non_member = create_user(&db, "s3h-nm@test.local").await.expect("nm");
        let tid = personal_team_id(&db, &owner).await.expect("tid");
        configure_personal_team_members(
            &db,
            &owner,
            &tid,
            vec![
                (cm.id.clone(), TeamRole::ContentMaintainer),
                (guest_u.id.clone(), TeamRole::Guest),
            ],
        )
        .await
        .expect("acl");
        (db, owner, cm, guest_u, non_member, tid)
    }

    /// BLC-SONG-002, BLC-SONG-006: non-member reads song → NotFound (verify it is not 403).
    #[tokio::test]
    async fn blc_song_002_non_member_read_not_found() {
        let (db, owner, _cm, _guest, nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let nm_p = UserPermissions::from_ref(&nm, &svc.teams);
        let song = create_song_with_title(&db, &owner, "NMSong")
            .await
            .expect("song");
        let r = svc.get_song_for_user(&nm_p, &song.id).await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
        // Verify owner can still read it (sanity check that song exists).
        svc.get_song_for_user(&owner_p, &song.id)
            .await
            .expect("owner reads ok");
    }

    /// BLC-SONG-007: guest cannot PUT (update) a song.
    #[tokio::test]
    async fn blc_song_007_guest_cannot_put() {
        use shared::song::CreateSong;
        let (db, owner, _cm, guest_u, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let _owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let guest_p = UserPermissions::from_ref(&guest_u, &svc.teams);
        let song = create_song_with_title(&db, &owner, "GuestPUTSong")
            .await
            .expect("song");
        let create = CreateSong {
            not_a_song: false,
            blobs: vec![],
            data: crate::test_helpers::minimal_song_data(),
        };
        let r = svc.update_song_for_user(&guest_p, &song.id, create).await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
    }

    /// BLC-SONG-007: guest cannot DELETE a song.
    #[tokio::test]
    async fn blc_song_007_guest_cannot_delete() {
        let (db, owner, _cm, guest_u, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let guest_p = UserPermissions::from_ref(&guest_u, &svc.teams);
        let song = create_song_with_title(&db, &owner, "GuestDELSong")
            .await
            .expect("song");
        let r = svc.delete_song_for_user(&guest_p, &song.id).await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
        // Song still exists.
        svc.get_song_for_user(&owner_p, &song.id)
            .await
            .expect("still exists");
    }

    /// BLC-SONG-008: content_maintainer can update a song.
    #[tokio::test]
    async fn blc_song_008_content_maintainer_can_update() {
        use shared::song::CreateSong;
        let (db, owner, cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let _owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let cm_p = UserPermissions::from_ref(&cm, &svc.teams);
        let song = create_song_with_title(&db, &owner, "CMUpdateSong")
            .await
            .expect("song");
        let mut data = crate::test_helpers::minimal_song_data();
        data.titles = vec!["UpdatedTitle".into()];
        let create = CreateSong {
            not_a_song: false,
            blobs: vec![],
            data,
        };
        svc.update_song_for_user(&cm_p, &song.id, create)
            .await
            .expect("cm update")
            .into_song();
    }

    /// BLC-SONG-003: PUT does not change the song's owner.
    #[tokio::test]
    async fn blc_song_003_put_does_not_change_owner() {
        use shared::song::CreateSong;
        let (db, owner, _cm, _guest, _nm, tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let song = create_song_with_title(&db, &owner, "OwnerSong")
            .await
            .expect("song");
        assert_eq!(song.owner, tid);
        let data = crate::test_helpers::minimal_song_data();
        let create = CreateSong {
            not_a_song: false,
            blobs: vec![],
            data,
        };
        let updated = svc
            .update_song_for_user(&owner_p, &song.id, create)
            .await
            .expect("update")
            .into_song();
        assert_eq!(updated.owner, tid, "owner must not change on PUT");
    }

    /// BLC-SONG-011: list songs filtered by artist name matches.
    #[tokio::test]
    async fn blc_song_011_search_by_artist() {
        use shared::api::ListQuery;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);

        let mut data_with_artist = crate::test_helpers::minimal_song_data();
        data_with_artist.titles = vec!["SongByArtist".into()];
        data_with_artist.artists = vec!["UniqueArtistZZZ".into()];
        let create = shared::song::CreateSong {
            not_a_song: false,
            blobs: vec![],
            data: data_with_artist,
        };
        svc.create_song_for_user(&owner_p, create)
            .await
            .expect("with artist");

        let mut data_no_artist = crate::test_helpers::minimal_song_data();
        data_no_artist.titles = vec!["SongWithoutArtist".into()];
        let create2 = shared::song::CreateSong {
            not_a_song: false,
            blobs: vec![],
            data: data_no_artist,
        };
        svc.create_song_for_user(&owner_p, create2)
            .await
            .expect("without artist");

        let results = svc
            .list_songs_for_user(&owner_p, ListQuery::new().with_q("UniqueArtistZZZ").into())
            .await
            .expect("search artist");
        assert_eq!(results.len(), 1, "only the song with the artist must match");
        assert_eq!(results[0].data.artists, vec!["UniqueArtistZZZ"]);
    }

    /// BLC-SONG-012: GET song includes `liked: true` when the caller has liked it.
    #[tokio::test]
    async fn blc_song_012_liked_true_when_liked() {
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let song = create_song_with_title(&db, &owner, "LikeSong")
            .await
            .expect("song");
        svc.set_song_like_status_for_user(&owner_p, &song.id, true)
            .await
            .expect("like");
        let fetched = svc
            .get_song_for_user(&owner_p, &song.id)
            .await
            .expect("get");
        assert!(fetched.user_specific_addons.liked);
    }

    /// BLC-SONG-012: GET song includes `liked: false` when not liked.
    #[tokio::test]
    async fn blc_song_012_liked_false_when_not_liked() {
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let song = create_song_with_title(&db, &owner, "UnlikedSong")
            .await
            .expect("song");
        let fetched = svc
            .get_song_for_user(&owner_p, &song.id)
            .await
            .expect("get");
        assert!(!fetched.user_specific_addons.liked);
    }

    /// BLC-SONG-004: user A likes song, user B (guest) does not; each sees independent state.
    #[tokio::test]
    async fn blc_song_004_like_state_independent_per_user() {
        let (db, owner, _cm, guest_u, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let guest_p = UserPermissions::from_ref(&guest_u, &svc.teams);
        let song = create_song_with_title(&db, &owner, "IndependentLike")
            .await
            .expect("song");
        svc.set_song_like_status_for_user(&owner_p, &song.id, true)
            .await
            .expect("owner likes");
        let owner_status = svc
            .song_like_status_for_user(&owner_p, &song.id)
            .await
            .expect("owner status");
        let guest_status = svc
            .song_like_status_for_user(&guest_p, &song.id)
            .await
            .expect("guest status");
        assert!(owner_status.liked, "owner must see liked=true");
        assert!(
            !guest_status.liked,
            "guest must see liked=false (they never liked)"
        );
    }

    /// BLC-SONG-004: like on a song the user cannot read returns NotFound.
    #[tokio::test]
    async fn blc_song_004_like_unreadable_song_not_found() {
        let (db, owner, _cm, _guest, nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let _owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let nm_p = UserPermissions::from_ref(&nm, &svc.teams);
        let song = create_song_with_title(&db, &owner, "SecretLikeSong")
            .await
            .expect("song");
        let r = svc
            .set_song_like_status_for_user(&nm_p, &song.id, true)
            .await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
    }

    /// BLC-SONG-018: PUT with a brand-new ID as owner creates the song (upsert).
    #[tokio::test]
    async fn blc_song_018_put_new_id_creates_song() {
        use shared::song::CreateSong;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let data = crate::test_helpers::minimal_song_data();
        let create = CreateSong {
            not_a_song: false,
            blobs: vec![],
            data,
        };
        let result = svc
            .update_song_for_user(&owner_p, "brand-new-id", create)
            .await;
        assert!(
            result.is_ok(),
            "upsert with new id must succeed for owner: {result:?}"
        );
    }

    /// PATCH-SONG-001: partial update only changes supplied fields; omitted fields keep their values.
    #[tokio::test]
    async fn patch_song_partial_update_only_changes_supplied_fields() {
        use shared::song::PatchSong;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);

        let song = create_song_with_title(&db, &owner, "OriginalTitle")
            .await
            .expect("song");
        assert!(!song.not_a_song);

        // Patch only not_a_song; title (data) must remain unchanged.
        let patched = svc
            .patch_song_for_user(
                &owner_p,
                &song.id,
                PatchSong {
                    not_a_song: Some(true),
                    blobs: None,
                    data: None,
                },
            )
            .await
            .expect("patch");

        assert!(patched.not_a_song, "not_a_song must be updated");
        assert_eq!(
            patched.data.title(),
            song.data.title(),
            "title must remain unchanged"
        );
        assert_eq!(patched.blobs, song.blobs, "blobs must remain unchanged");
    }

    /// PATCH-SONG-002: guest cannot PATCH a song (same ACL as PUT).
    #[tokio::test]
    async fn patch_song_guest_cannot_patch() {
        use shared::song::PatchSong;
        let (db, owner, _cm, guest_u, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let guest_p = UserPermissions::from_ref(&guest_u, &svc.teams);
        let song = create_song_with_title(&db, &owner, "GuestPatchSong")
            .await
            .expect("song");
        let r = svc
            .patch_song_for_user(
                &guest_p,
                &song.id,
                PatchSong {
                    not_a_song: Some(true),
                    blobs: None,
                    data: None,
                },
            )
            .await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
    }

    /// PATCH-SONG-003: PATCH on non-existent song returns NotFound.
    #[tokio::test]
    async fn patch_song_not_found() {
        use shared::song::PatchSong;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let r = svc
            .patch_song_for_user(
                &owner_p,
                "never-existed-song",
                PatchSong {
                    not_a_song: Some(true),
                    blobs: None,
                    data: None,
                },
            )
            .await;
        assert!(matches!(r, Err(crate::error::AppError::NotFound(_))));
    }

    /// PATCH-SONG-004: empty PATCH body leaves all fields unchanged.
    #[tokio::test]
    async fn patch_song_empty_body_is_noop() {
        use shared::song::PatchSong;
        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);
        let song = create_song_with_title(&db, &owner, "NoopSong")
            .await
            .expect("song");
        let patched = svc
            .patch_song_for_user(
                &owner_p,
                &song.id,
                PatchSong {
                    not_a_song: None,
                    blobs: None,
                    data: None,
                },
            )
            .await
            .expect("noop patch");
        assert_eq!(patched.not_a_song, song.not_a_song);
        assert_eq!(patched.data.title(), song.data.title());
    }

    #[tokio::test]
    async fn patch_song_all_field_combinations() {
        use shared::song::{CreateSong, PatchSong, PatchSongData};

        let (db, owner, _cm, _guest, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let owner_p = UserPermissions::from_ref(&owner, &svc.teams);

        let mut base_data = crate::test_helpers::minimal_song_data();
        base_data.titles = vec!["BaseTitle".into()];
        let patch_data = PatchSongData {
            titles: Some(vec!["PatchedTitle".into()]),
            ..PatchSongData::default()
        };

        for mask in 0u8..8 {
            let created = svc
                .create_song_for_user(
                    &owner_p,
                    CreateSong {
                        not_a_song: false,
                        blobs: vec!["base_blob".into()],
                        data: base_data.clone(),
                    },
                )
                .await
                .expect("create");

            let include_not_a_song = (mask & 0b001) != 0;
            let include_blobs = (mask & 0b010) != 0;
            let include_data = (mask & 0b100) != 0;
            let expected_not_a_song = include_not_a_song;
            let expected_blobs = if include_blobs {
                vec!["patched_blob".to_string()]
            } else {
                vec!["base_blob".to_string()]
            };
            let expected_title = if include_data {
                "PatchedTitle"
            } else {
                "BaseTitle"
            };

            let patched = svc
                .patch_song_for_user(
                    &owner_p,
                    &created.id,
                    PatchSong {
                        not_a_song: include_not_a_song.then_some(true),
                        blobs: include_blobs.then_some(vec!["patched_blob".into()]),
                        data: include_data.then_some(patch_data.clone()),
                    },
                )
                .await
                .expect("patch");

            assert_eq!(
                patched.not_a_song, expected_not_a_song,
                "mask={mask:03b}: not_a_song mismatch"
            );
            assert_eq!(
                patched.blobs, expected_blobs,
                "mask={mask:03b}: blobs mismatch"
            );
            assert_eq!(
                patched.data.title(),
                expected_title,
                "mask={mask:03b}: data.title mismatch"
            );
        }
    }

    #[test]
    fn patch_song_data_all_field_combinations() {
        use std::collections::BTreeMap;

        use chordlib::types::SimpleChord;
        use shared::patch::Patch;
        use shared::song::PatchSongData;

        let c: SimpleChord = SimpleChord::new(3);
        let d: SimpleChord = SimpleChord::new(5);

        let mut base_tags = BTreeMap::new();
        base_tags.insert("base".to_string(), "value".to_string());
        let mut patch_tags = BTreeMap::new();
        patch_tags.insert("patched".to_string(), "yes".to_string());

        let base = chordlib::types::Song {
            titles: vec!["base-title".into()],
            subtitle: Some("base-sub".into()),
            copyright: Some("base-copyright".into()),
            key: Some(c.clone()),
            artists: vec!["base-artist".into()],
            languages: vec!["en".into()],
            tempo: Some(100),
            time: Some((4, 4)),
            tags: base_tags.clone(),
            sections: vec![],
        };

        for mask in 0u16..1024 {
            let include_titles = (mask & (1 << 0)) != 0;
            let include_subtitle = (mask & (1 << 1)) != 0;
            let include_copyright = (mask & (1 << 2)) != 0;
            let include_key = (mask & (1 << 3)) != 0;
            let include_artists = (mask & (1 << 4)) != 0;
            let include_languages = (mask & (1 << 5)) != 0;
            let include_tempo = (mask & (1 << 6)) != 0;
            let include_time = (mask & (1 << 7)) != 0;
            let include_tags = (mask & (1 << 8)) != 0;
            let include_sections = (mask & (1 << 9)) != 0;

            let patch = PatchSongData {
                titles: include_titles.then_some(vec!["patched-title".into()]),
                subtitle: if include_subtitle {
                    Patch::Value("patched-sub".into())
                } else {
                    Patch::Missing
                },
                copyright: if include_copyright {
                    Patch::Value("patched-copyright".into())
                } else {
                    Patch::Missing
                },
                key: if include_key {
                    Patch::Value(d.clone())
                } else {
                    Patch::Missing
                },
                artists: include_artists.then_some(vec!["patched-artist".into()]),
                languages: include_languages.then_some(vec!["de".into()]),
                tempo: if include_tempo {
                    Patch::Value(128)
                } else {
                    Patch::Missing
                },
                time: if include_time {
                    Patch::Value((3, 4))
                } else {
                    Patch::Missing
                },
                tags: include_tags.then_some(patch_tags.clone()),
                sections: include_sections.then_some(vec![]),
            };

            let merged = SongServiceHandle::merge_song_data(base.clone(), patch);

            assert_eq!(
                merged.titles,
                if include_titles {
                    vec!["patched-title".to_string()]
                } else {
                    vec!["base-title".to_string()]
                },
                "mask={mask:010b}: titles mismatch"
            );
            assert_eq!(
                merged.subtitle.as_deref(),
                Some(if include_subtitle {
                    "patched-sub"
                } else {
                    "base-sub"
                }),
                "mask={mask:010b}: subtitle mismatch"
            );
            assert_eq!(
                merged.copyright.as_deref(),
                Some(if include_copyright {
                    "patched-copyright"
                } else {
                    "base-copyright"
                }),
                "mask={mask:010b}: copyright mismatch"
            );
            assert_eq!(
                merged.key,
                Some(if include_key { d.clone() } else { c.clone() }),
                "mask={mask:010b}: key mismatch"
            );
            assert_eq!(
                merged.artists,
                if include_artists {
                    vec!["patched-artist".to_string()]
                } else {
                    vec!["base-artist".to_string()]
                },
                "mask={mask:010b}: artists mismatch"
            );
            assert_eq!(
                merged.languages,
                if include_languages {
                    vec!["de".to_string()]
                } else {
                    vec!["en".to_string()]
                },
                "mask={mask:010b}: languages mismatch"
            );
            assert_eq!(
                merged.tempo,
                Some(if include_tempo { 128 } else { 100 }),
                "mask={mask:010b}: tempo mismatch"
            );
            assert_eq!(
                merged.time,
                Some(if include_time { (3, 4) } else { (4, 4) }),
                "mask={mask:010b}: time mismatch"
            );
            assert_eq!(
                merged.tags,
                if include_tags {
                    patch_tags.clone()
                } else {
                    base_tags.clone()
                },
                "mask={mask:010b}: tags mismatch"
            );
            assert_eq!(
                merged.sections,
                vec![],
                "mask={mask:010b}: sections mismatch"
            );
        }
    }

    #[test]
    fn patch_song_data_null_clears_nullable_fields() {
        use shared::patch::Patch;
        use shared::song::PatchSongData;

        let c: chordlib::types::SimpleChord = chordlib::types::SimpleChord::new(3);
        let base = chordlib::types::Song {
            titles: vec!["base-title".into()],
            subtitle: Some("base-sub".into()),
            copyright: Some("base-copyright".into()),
            key: Some(c),
            artists: vec!["base-artist".into()],
            languages: vec!["en".into()],
            tempo: Some(100),
            time: Some((4, 4)),
            tags: Default::default(),
            sections: vec![],
        };

        let merged = SongServiceHandle::merge_song_data(
            base,
            PatchSongData {
                subtitle: Patch::Null,
                copyright: Patch::Null,
                key: Patch::Null,
                tempo: Patch::Null,
                time: Patch::Null,
                ..PatchSongData::default()
            },
        );

        assert_eq!(merged.subtitle, None);
        assert_eq!(merged.copyright, None);
        assert_eq!(merged.key, None);
        assert_eq!(merged.tempo, None);
        assert_eq!(merged.time, None);
    }

    /// PATCH-SONG-018: PUT with a brand-new ID as guest on someone else's team creates the song on
    /// the caller's own personal team (upsert; owner determined by caller, not team membership).
    #[tokio::test]
    async fn blc_song_018_put_new_id_as_guest_creates_on_own_team() {
        use crate::test_helpers::personal_team_id;
        use shared::song::CreateSong;
        let (db, _owner, _cm, guest_u, _nm, _tid) = four_user_song_fixture().await;
        let svc = SongServiceHandle::build(db.clone());
        let guest_p = UserPermissions::from_ref(&guest_u, &svc.teams);
        let data = crate::test_helpers::minimal_song_data();
        let create = CreateSong {
            not_a_song: false,
            blobs: vec![],
            data,
        };
        // Guest can create songs on their own personal team via upsert.
        let result = svc
            .update_song_for_user(&guest_p, "brand-new-guest-created-id", create)
            .await
            .expect("guest can upsert to own personal team")
            .into_song();
        let guest_pt = personal_team_id(&db, &guest_u).await.expect("guest pt");
        assert_eq!(
            result.owner, guest_pt,
            "upserted song must be owned by guest's personal team"
        );
    }

    /// BLC-SONG-010, BLC-COLL-016: user without default_collection → "Default" collection created.
    #[tokio::test]
    async fn blc_song_010_no_default_collection_creates_default() {
        use shared::api::ListQuery;
        let db = test_db().await.expect("db");
        let u = create_user(&db, "s3i-new@test.local").await.expect("u");
        assert!(
            u.default_collection.is_none(),
            "new user must have no default_collection"
        );
        let svc = SongServiceHandle::build(db.clone());
        let perms = UserPermissions::from_ref(&u, &svc.teams);
        let song = svc
            .create_song_for_user(
                &perms,
                shared::song::CreateSong {
                    not_a_song: false,
                    blobs: vec![],
                    data: crate::test_helpers::minimal_song_data(),
                },
            )
            .await
            .expect("create song");

        // A "Default" collection must have been created.
        let coll_svc = crate::test_helpers::collection_service(&db);
        let fresh_perms = UserPermissions::from_ref(&u, &coll_svc.teams);
        let collections = coll_svc
            .list_collections_for_user(&fresh_perms, ListQuery::default())
            .await
            .expect("collections");
        let default_coll = collections.iter().find(|c| c.title == "Default");
        assert!(
            default_coll.is_some(),
            "a 'Default' collection must be created"
        );

        // The "Default" collection must contain the newly created song.
        let coll = default_coll.unwrap();
        let (songs, _) = coll_svc
            .collection_songs_for_user(&fresh_perms, &coll.id, ListQuery::default())
            .await
            .expect("songs");
        assert!(
            songs.iter().any(|s| s.id == song.id),
            "song must be in the Default collection"
        );

        // The user's default_collection field must be updated.
        let user_svc = crate::test_helpers::user_service(&db);
        let updated_user = user_svc.get_user(&u.id).await.expect("get user");
        assert_eq!(
            updated_user.default_collection.as_deref(),
            Some(coll.id.as_str()),
            "user.default_collection must point to the Default collection"
        );
    }

    /// BLC-SONG-010: user with existing default_collection → song appended to it.
    #[tokio::test]
    async fn blc_song_010_with_default_collection_appends() {
        use shared::api::ListQuery;
        let db = test_db().await.expect("db");
        // Create user and first song (auto-creates Default collection + sets default_collection).
        let u = create_user(&db, "s3i-existing@test.local")
            .await
            .expect("u");
        let svc = SongServiceHandle::build(db.clone());
        let perms = UserPermissions::from_ref(&u, &svc.teams);
        let song1 = svc
            .create_song_for_user(
                &perms,
                shared::song::CreateSong {
                    not_a_song: false,
                    blobs: vec![],
                    data: {
                        let mut d = crate::test_helpers::minimal_song_data();
                        d.titles = vec!["First".into()];
                        d
                    },
                },
            )
            .await
            .expect("song1");

        // Fetch the updated user (who now has default_collection set).
        let user_svc = crate::test_helpers::user_service(&db);
        let updated_u = user_svc.get_user(&u.id).await.expect("user");
        assert!(
            updated_u.default_collection.is_some(),
            "default_collection should be set"
        );

        // Create a second song using the updated user (whose default_collection is set).
        let svc2 = SongServiceHandle::build(db.clone());
        let perms2 = UserPermissions::from_ref(&updated_u, &svc2.teams);
        let song2 = svc2
            .create_song_for_user(
                &perms2,
                shared::song::CreateSong {
                    not_a_song: false,
                    blobs: vec![],
                    data: {
                        let mut d = crate::test_helpers::minimal_song_data();
                        d.titles = vec!["Second".into()];
                        d
                    },
                },
            )
            .await
            .expect("song2");

        let coll_svc = crate::test_helpers::collection_service(&db);
        let fresh_perms = UserPermissions::from_ref(&updated_u, &coll_svc.teams);
        let collections = coll_svc
            .list_collections_for_user(&fresh_perms, ListQuery::default())
            .await
            .expect("collections");
        assert_eq!(
            collections.len(),
            1,
            "must still have exactly one collection"
        );
        let (songs, _) = coll_svc
            .collection_songs_for_user(&fresh_perms, &collections[0].id, ListQuery::default())
            .await
            .expect("songs");
        let song_ids: Vec<&str> = songs.iter().map(|s| s.id.as_str()).collect();
        assert!(
            song_ids.contains(&song1.id.as_str()),
            "song1 must be in Default collection"
        );
        assert!(
            song_ids.contains(&song2.id.as_str()),
            "song2 must be in Default collection"
        );
    }

    #[tokio::test]
    async fn blc_song_delete_after_setlist_link() {
        let db = test_db().await.expect("db");
        let svc = SongServiceHandle::build(db.clone());

        let u = create_user(&db, "song-del@test.local").await.expect("u");
        let song = create_song_with_title(&db, &u, "ToDelete")
            .await
            .expect("song");
        let sl_svc = setlist_service(&db);
        let u_p = UserPermissions::from_ref(&u, &sl_svc.teams);
        let sl = sl_svc
            .create_setlist_for_user(
                &u_p,
                setlist_with_songs("L", &[(song.id.as_str(), Some("1"))]),
            )
            .await
            .expect("setlist");
        let song_p = UserPermissions::from_ref(&u, &svc.teams);
        svc.delete_song_for_user(&song_p, &song.id)
            .await
            .expect("del song");
        let sl_svc2 = setlist_service(&db);
        let u_p2 = UserPermissions::from_ref(&u, &sl_svc2.teams);
        let g = sl_svc2
            .get_setlist_for_user(&u_p2, &sl.id)
            .await
            .expect("get setlist");
        assert!(g.songs.is_empty());
    }
}
