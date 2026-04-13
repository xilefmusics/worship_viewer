use actix_web::{HttpResponse, web::Data};
use shared::api::ListQuery;
use shared::player::Player;
use shared::setlist::{CreateSetlist, Setlist};
use shared::song::Song;

use crate::error::AppError;
use crate::resources::User;
use crate::resources::song::LikedSongIds;
use crate::resources::song::{Format, export};
use crate::resources::team::TeamResolver;

use super::model::player_from_song_links;
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
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        self.repo.get_setlists(read_teams, pagination).await
    }

    pub async fn get_setlist_for_user(&self, user: &User, id: &str) -> Result<Setlist, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
        self.repo.get_setlist(read_teams, id).await
    }

    pub async fn setlist_player_for_user(&self, user: &User, id: &str) -> Result<Player, AppError> {
        let liked_set = self.likes.liked_song_ids(&user.id).await?;
        let read_teams = self.teams.content_read_teams(user).await?;
        let links = self.repo.get_setlist_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn export_setlist_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = self.teams.content_read_teams(user).await?;
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
        user: &User,
        id: &str,
    ) -> Result<Vec<Song>, AppError> {
        let liked_set = self.likes.liked_song_ids(&user.id).await?;
        let read_teams = self.teams.content_read_teams(user).await?;
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
        user: &User,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        self.repo.create_setlist(&user.id, setlist).await
    }

    pub async fn update_setlist_for_user(
        &self,
        user: &User,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
        self.repo.update_setlist(write_teams, id, setlist).await
    }

    pub async fn delete_setlist_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Setlist, AppError> {
        let write_teams = self.teams.content_write_teams(user).await?;
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
    use crate::resources::team::TeamResolver;

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
        let r = svc.get_setlist_for_user(&test_user(), "nope").await;
        assert!(matches!(r, Err(AppError::NotFound(_))));
    }

    /// BLC-SETL-007: write teams exclude owner → update NotFound
    #[tokio::test]
    async fn update_rejects_when_user_not_in_write_teams() {
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
        let r = svc
            .update_setlist_for_user(
                &test_user(),
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
        let r = svc
            .update_setlist_for_user(
                &test_user(),
                "id",
                CreateSetlist {
                    title: "t".into(),
                    songs: vec![],
                },
            )
            .await;
        assert!(r.is_ok());
    }
}
