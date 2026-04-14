use actix_web::{HttpResponse, web::Data};
use async_trait::async_trait;

use shared::api::ListQuery;
use shared::like::LikeStatus;
use shared::player::Player;
use shared::song::{CreateSong, Link as SongLink, LinkOwned as SongLinkOwned, Song};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::collection::CollectionRepository;

use crate::resources::team::TeamResolver;
use crate::resources::UserModel;
use shared::collection::CreateCollection;

use super::export::{Format, export};
use super::liked::LikedSongIds;
use super::repository::SongRepository;
use super::surreal_repo::SurrealSongRepo;

/// Abstraction over updating a user's default collection reference.
#[async_trait]
pub trait UserCollectionUpdater: Send + Sync {
    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl UserCollectionUpdater for Data<Database> {
    async fn set_default_collection(
        &self,
        user_id: &str,
        collection_id: &str,
    ) -> Result<(), AppError> {
        UserModel::set_default_collection_to_user(self.get_ref(), user_id, collection_id).await
    }
}

/// Application service: team resolution, authorization, and orchestration for songs.
#[derive(Clone)]
pub struct SongService<R, T, L, C, U> {
    pub repo: R,
    pub teams: T,
    pub likes: L,
    pub collections: C,
    pub user_updater: U,
}

impl<R, T, L, C, U> SongService<R, T, L, C, U> {
    pub fn new(repo: R, teams: T, likes: L, collections: C, user_updater: U) -> Self {
        Self { repo, teams, likes, collections, user_updater }
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
    pub async fn list_songs_for_user(
        &self,
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user.id),
            self.teams.content_read_teams(user)
        )?;
        Ok(self
            .repo
            .get_songs(read_teams, pagination)
            .await?
            .into_iter()
            .map(|mut song| {
                song.user_specific_addons.liked = liked_set.contains(&song.id);
                song
            })
            .collect())
    }

    pub async fn get_song_for_user(&self, user: &User, id: &str) -> Result<Song, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user.id),
            self.teams.content_read_teams(user)
        )?;
        let mut song = self.repo.get_song(read_teams, id).await?;
        song.user_specific_addons.liked = liked_set.contains(&song.id);
        Ok(song)
    }

    pub async fn song_player_for_user(&self, user: &User, id: &str) -> Result<Player, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        Ok(Player::from(SongLinkOwned {
            song: self.repo.get_song(read_teams.clone(), id).await?,
            nr: None,
            key: None,
            liked: self.repo.get_song_like(read_teams, &user.id, id).await?,
        }))
    }

    pub async fn export_song_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        let song = self.repo.get_song(read_teams, id).await?;
        export(vec![song], format).await
    }

    pub async fn create_song_for_user(
        &self,
        user: &User,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let created = self.repo.create_song(&user.id, song).await?;

        if let Some(collection_id) = user.default_collection.as_ref() {
            let write_teams = self.teams.content_write_teams(user).await?;
            self.collections
                .add_song_to_collection(
                    write_teams,
                    collection_id,
                    SongLink {
                        id: created.id.clone(),
                        nr: None,
                        key: None,
                    },
                )
                .await?;
        } else {
            let collection = self
                .collections
                .create_collection(
                    &user.id,
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
                .set_default_collection(&user.id, &collection.id)
                .await?;
        }

        Ok(created)
    }

    pub async fn update_song_for_user(
        &self,
        user: &User,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
        self.repo.update_song(write_teams, &user.id, id, song).await
    }

    pub async fn delete_song_for_user(&self, user: &User, id: &str) -> Result<Song, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
        self.repo.delete_song(write_teams, id).await
    }

    pub async fn song_like_status_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        let liked = self.repo.get_song_like(read_teams, &user.id, id).await?;
        Ok(LikeStatus { liked })
    }

    pub async fn set_song_like_status_for_user(
        &self,
        user: &User,
        id: &str,
        liked: bool,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        let liked = self.repo.set_song_like(read_teams, &user.id, id, liked).await?;
        Ok(LikeStatus { liked })
    }
}

/// Production type alias used in HTTP wiring.
pub type SongServiceHandle = SongService<
    SurrealSongRepo,
    crate::resources::team::SurrealTeamResolver,
    Data<Database>,
    crate::resources::collection::SurrealCollectionRepo,
    Data<Database>,
>;

impl SongServiceHandle {
    pub fn build(db: Data<Database>) -> Self {
        SongService::new(
            SurrealSongRepo::new(db.clone()),
            crate::resources::team::SurrealTeamResolver::new(db.clone()),
            db.clone(),
            crate::resources::collection::SurrealCollectionRepo::new(db.clone()),
            db.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use actix_web::web::Data;

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
        let data = Data::from(db.clone());
        let svc = SongServiceHandle::build(data);

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

        let list = svc
            .list_songs_for_user(&owner, ListQuery::default())
            .await
            .expect("list");
        assert!(list.len() >= 2);

        let q = svc
            .list_songs_for_user(&owner, ListQuery::new().with_q("Alpha"))
            .await
            .expect("search");
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].id, s1.id);

        svc.get_song_for_user(&owner, &s1.id).await.expect("get");
        svc.get_song_for_user(&other, &s1.id)
            .await
            .expect("guest read");

        let bad = svc.get_song_for_user(&owner, "setlist:not-a-song").await;
        assert!(bad.is_err(), "wrong table id should not resolve: {bad:?}");

        svc.set_song_like_status_for_user(&owner, &s1.id, true)
            .await
            .expect("like");
        let st = svc
            .song_like_status_for_user(&owner, &s1.id)
            .await
            .expect("like status");
        assert!(st.liked);

        svc.delete_song_for_user(&owner, &s1.id).await.expect("del");
    }

    #[tokio::test]
    async fn blc_song_delete_after_setlist_link() {
        let db = test_db().await.expect("db");
        let data = Data::from(db.clone());
        let svc = SongServiceHandle::build(data);

        let u = create_user(&db, "song-del@test.local").await.expect("u");
        let song = create_song_with_title(&db, &u, "ToDelete")
            .await
            .expect("song");
        let sl = setlist_service(&db)
            .create_setlist_for_user(
                &u,
                setlist_with_songs("L", &[(song.id.as_str(), Some("1"))]),
            )
            .await
            .expect("setlist");
        svc.delete_song_for_user(&u, &song.id)
            .await
            .expect("del song");
        let g = setlist_service(&db)
            .get_setlist_for_user(&u, &sl.id)
            .await
            .expect("get setlist");
        assert!(g.songs.is_empty());
    }
}
