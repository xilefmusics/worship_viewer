use actix_web::{HttpResponse, web::Data};

use shared::api::ListQuery;
use shared::collection::{Collection, CreateCollection};
use shared::player::Player;
use shared::song::Song;

use crate::database::Database;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::common::player_from_song_links;
use crate::resources::song::{Format, LikedSongIds, export};
use crate::resources::team::TeamResolver;

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
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        self.repo.get_collections(read_teams, pagination).await
    }

    pub async fn get_collection_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Collection, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        self.repo.get_collection(read_teams, id).await
    }

    pub async fn collection_player_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Player, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user.id),
            self.teams.content_read_teams(user)
        )?;
        let links = self.repo.get_collection_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn export_collection_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        let songs: Vec<Song> = self
            .repo
            .get_collection_songs(read_teams, id)
            .await?
            .into_iter()
            .map(|l| l.song)
            .collect();
        export(songs, format).await
    }

    pub async fn collection_songs_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user.id),
            self.teams.content_read_teams(user)
        )?;
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
        user: &User,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        self.repo.create_collection(&user.id, collection).await
    }

    pub async fn update_collection_for_user(
        &self,
        user: &User,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
        self.repo.update_collection(write_teams, id, collection).await
    }

    pub async fn delete_collection_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Collection, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
        self.repo.delete_collection(write_teams, id).await
    }
}

/// Production type alias used in HTTP wiring.
pub type CollectionServiceHandle = CollectionService<
    SurrealCollectionRepo,
    crate::resources::team::SurrealTeamResolver,
    Data<Database>,
>;

impl CollectionServiceHandle {
    pub fn build(db: Data<Database>) -> Self {
        CollectionService::new(
            SurrealCollectionRepo::new(db.clone()),
            crate::resources::team::SurrealTeamResolver::new(db.clone()),
            db.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use actix_web::web::Data;
    use shared::song::Link as SongLink;

    use crate::error::AppError;
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
        let data = Data::from(db.clone());
        let svc = CollectionServiceHandle::build(data);

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

        let col = svc
            .create_collection_for_user(
                &owner,
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
            .list_collections_for_user(&owner, ListQuery::default())
            .await
            .expect("list");
        assert!(list.iter().any(|c| c.id == col.id));

        svc.get_collection_for_user(&guest, &col.id)
            .await
            .expect("guest read");

        let upd = svc
            .update_collection_for_user(
                &owner,
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
                &guest,
                &col.id,
                CreateCollection {
                    title: "Nope".into(),
                    cover: "mysongs".into(),
                    songs: vec![],
                },
            )
            .await;
        assert!(matches!(put_guest, Err(AppError::NotFound(_))));

        svc.delete_collection_for_user(&owner, &col.id)
            .await
            .expect("delete");
    }
}
