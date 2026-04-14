use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    api::ListQuery,
    collection::{Collection, CreateCollection},
    player::Player,
    song::{Link as SongLink, LinkOwned as SongLinkOwned, Song},
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::common::{
    FetchedSongRecord, SongLinkRecord, belongs_to, blob_thing, player_from_song_links, resource_id,
};
use crate::resources::song::{Format, Model as SongDbModel, export};
use crate::resources::team::{content_read_team_things, content_write_team_things};

pub trait Model {
    async fn get_collections(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError>;
    async fn get_collection(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Collection, AppError>;
    async fn get_collection_songs(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError>;
    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn update_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError>;
    async fn delete_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Collection, AppError>;
    async fn add_song_to_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError>;
}

impl Model for Database {
    async fn get_collections(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError> {
        let q_nonempty = pagination.q.as_ref().is_some_and(|q| !q.trim().is_empty());
        let mut query = if q_nonempty {
            String::from(
                "SELECT *, (search::score(0) ?? 0) AS score FROM collection WHERE owner IN $teams",
            )
        } else {
            String::from("SELECT * FROM collection WHERE owner IN $teams")
        };
        if q_nonempty {
            query.push_str(" AND title @0@ $q ORDER BY score DESC");
        }
        if pagination.to_offset_limit().is_some() {
            query.push_str(" LIMIT $limit START $start");
        }

        let mut request = self.db.query(query).bind(("teams", read_teams));
        if let Some(ref q) = pagination.q
            && !q.trim().is_empty()
        {
            request = request.bind(("q", q.trim().to_string()));
        }
        if let Some((offset, limit)) = pagination.to_offset_limit() {
            request = request.bind(("limit", limit)).bind(("start", offset));
        }

        let mut response = request.await?;

        Ok(response
            .take::<Vec<CollectionRecord>>(0)?
            .into_iter()
            .map(CollectionRecord::into_collection)
            .collect())
    }

    async fn get_collection(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let record: Option<CollectionRecord> =
            self.db.select(resource_id("collection", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, &read_teams) => Ok(r.into_collection()),
            _ => Err(AppError::NotFound("collection not found".into())),
        }
    }

    async fn get_collection_songs(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError> {
        let resource = resource_id("collection", id)?;
        let mut response = self
            .db
            .query("SELECT owner, songs FROM collection WHERE id = $id FETCH songs.*.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<CollectionSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !belongs_to(&record.owner, &read_teams) {
            return Err(AppError::NotFound("collection not found".into()));
        }

        Ok(record.into_songs())
    }

    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let owner_team = self.personal_team_thing_for_user(owner).await?;
        self.db
            .create("collection")
            .content(CollectionRecord::from_payload(
                None,
                Some(owner_team),
                collection,
            ))
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to create collection"))
    }

    async fn update_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let (tb, sid) = resource_id("collection", id)?;
        let songs: Vec<SongLinkRecord> = collection.songs.into_iter().map(Into::into).collect();
        let cover = blob_thing(&collection.cover);
        let title = collection.title;

        let mut response = self
            .db
            .query(
                "UPDATE type::thing($tb, $sid) SET title = $title, cover = $cover, songs = $songs \
                 WHERE owner IN $teams RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("title", title))
            .bind(("cover", cover))
            .bind(("songs", songs))
            .bind(("teams", write_teams))
            .await?;

        let rows: Vec<CollectionRecord> = response.take(0)?;
        rows
            .into_iter()
            .next()
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn delete_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let (tb, sid) = resource_id("collection", id)?;
        let mut response = self
            .db
            .query(
                "DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams))
            .await?;

        let rows: Vec<CollectionRecord> = response.take(0)?;
        rows
            .into_iter()
            .next()
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn add_song_to_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError> {
        let _ = self
            .db
            .query(
                r#"
            UPDATE type::thing("collection", $id)
            SET songs = array::append(songs, $song)
            WHERE owner IN $teams;
            "#,
            )
            .bind(("id", id.to_owned()))
            .bind(("teams", write_teams))
            .bind(("song", SongLinkRecord::from(song_link)))
            .await?;

        Ok(())
    }
}

impl Database {
    pub async fn list_collections_for_user(
        &self,
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_collections(read_teams, pagination).await
    }

    pub async fn get_collection_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Collection, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_collection(read_teams, id).await
    }

    pub async fn collection_player_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Player, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            SongDbModel::get_liked_set(self, &user.id),
            content_read_team_things(self, user)
        )?;
        let links = self.get_collection_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn export_collection_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let songs: Vec<Song> = self
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
            SongDbModel::get_liked_set(self, &user.id),
            content_read_team_things(self, user)
        )?;
        Ok(self
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
        self.create_collection(&user.id, collection).await
    }

    pub async fn update_collection_for_user(
        &self,
        user: &User,
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.update_collection(write_teams, id, collection).await
    }

    pub async fn delete_collection_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Collection, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.delete_collection(write_teams, id).await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct CollectionRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    title: String,
    cover: Option<Thing>,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl CollectionRecord {
    fn into_collection(self) -> Collection {
        Collection {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            title: self.title,
            cover: self
                .cover
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, collection: CreateCollection) -> Self {
        Self {
            id,
            owner,
            title: collection.title,
            cover: Some(blob_thing(&collection.cover)),
            songs: collection.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
struct CollectionSongsRecord {
    #[serde(default)]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl CollectionSongsRecord {
    fn into_songs(self) -> Vec<SongLinkOwned> {
        self.songs
            .into_iter()
            .map(|record| record.into_song_link_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::song::Link as SongLink;

    #[test]
    fn collection_record_from_payload_into_collection() {
        let id = Thing::from(("collection".to_owned(), "c1".to_owned()));
        let owner = Thing::from(("team".to_owned(), "tm".to_owned()));
        let record = CollectionRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            CreateCollection {
                title: "Hits".into(),
                cover: "blob:cover1".into(),
                songs: vec![SongLink {
                    id: "s1".into(),
                    nr: None,
                    key: None,
                }],
            },
        );
        let c = record.into_collection();
        assert_eq!(c.id, "c1");
        assert_eq!(c.owner, "tm");
        assert_eq!(c.title, "Hits");
        assert_eq!(c.cover, "cover1");
        assert_eq!(c.songs.len(), 1);
        assert_eq!(c.songs[0].id, "s1");
    }

    #[tokio::test]
    async fn blc_collection_crud_and_acl() {
        use shared::api::ListQuery;
        use shared::team::TeamRole;

        use crate::error::AppError;
        use crate::test_helpers::{
            configure_personal_team_members, create_song_with_title, create_user,
            personal_team_id, test_db,
        };

        let db = test_db().await.expect("db");
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

        let col = db
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

        let list = db
            .list_collections_for_user(&owner, ListQuery::default())
            .await
            .expect("list");
        assert!(list.iter().any(|c| c.id == col.id));

        db.get_collection_for_user(&guest, &col.id)
            .await
            .expect("guest read");

        let upd = db
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

        let put_guest = db
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

        db.delete_collection_for_user(&owner, &col.id)
            .await
            .expect("delete");
    }
}
