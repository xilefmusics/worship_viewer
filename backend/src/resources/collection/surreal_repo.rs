use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::sql::Thing;

use serde::Deserialize;

use shared::api::ListQuery;
use shared::collection::{Collection, CreateCollection};
use shared::song::{Link as SongLink, LinkOwned as SongLinkOwned};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::{SongLinkRecord, belongs_to, blob_thing, resource_id};

use super::model::{CollectionRecord, CollectionSongsRecord};
use super::repository::CollectionRepository;

#[derive(Clone)]
pub struct SurrealCollectionRepo {
    db: Arc<Database>,
}

impl SurrealCollectionRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl CollectionRepository for SurrealCollectionRepo {
    async fn get_collections(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Collection>, AppError> {
        let db = self.inner();
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
        let (offset, limit) = pagination.effective_offset_limit();
        query.push_str(" LIMIT $limit START $start");

        let mut request = db.db.query(query).bind(("teams", read_teams.to_vec()));
        if let Some(ref q) = pagination.q
            && !q.trim().is_empty()
        {
            request = request.bind(("q", q.trim().to_string()));
        }
        request = request.bind(("limit", limit)).bind(("start", offset));

        let mut response = request.await?;

        Ok(response
            .take::<Vec<CollectionRecord>>(0)?
            .into_iter()
            .map(CollectionRecord::into_collection)
            .collect())
    }

    async fn count_collections(
        &self,
        read_teams: &[Thing],
        q: Option<&str>,
    ) -> Result<u64, AppError> {
        #[derive(Deserialize)]
        struct CountResult {
            count: u64,
        }
        let q_nonempty = q.is_some_and(|s| !s.trim().is_empty());
        let mut query = String::from("SELECT count() FROM collection WHERE owner IN $teams");
        if q_nonempty {
            query.push_str(" AND title @0@ $q");
        }
        query.push_str(" GROUP ALL");

        let mut request = self
            .inner()
            .db
            .query(query)
            .bind(("teams", read_teams.to_vec()));
        if q_nonempty {
            request = request.bind(("q", q.unwrap().trim().to_string()));
        }
        let mut response = request.await?;
        Ok(response
            .take::<Vec<CountResult>>(0)?
            .into_iter()
            .next()
            .map(|r| r.count)
            .unwrap_or(0))
    }

    async fn get_collection(&self, read_teams: &[Thing], id: &str) -> Result<Collection, AppError> {
        let db = self.inner();
        let record: Option<CollectionRecord> = db.db.select(resource_id("collection", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, read_teams) => Ok(r.into_collection()),
            _ => Err(AppError::NotFound("collection not found".into())),
        }
    }

    async fn get_collection_songs(
        &self,
        read_teams: &[Thing],
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError> {
        let db = self.inner();
        let resource = resource_id("collection", id)?;
        let mut response = db
            .db
            .query("SELECT owner, songs FROM collection WHERE id = $id FETCH songs.*.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<CollectionSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !belongs_to(&record.owner, read_teams) {
            return Err(AppError::NotFound("collection not found".into()));
        }

        Ok(record.into_songs())
    }

    async fn create_collection(
        &self,
        owner: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let db = self.inner();
        let owner_team = db.personal_team_thing_for_user(owner).await?;
        db.db
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
        write_teams: &[Thing],
        id: &str,
        collection: CreateCollection,
    ) -> Result<Collection, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("collection", id)?;
        let songs: Vec<SongLinkRecord> = collection.songs.into_iter().map(Into::into).collect();
        let cover = blob_thing(&collection.cover);
        let title = collection.title;

        let mut response = db
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
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<CollectionRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn delete_collection(
        &self,
        write_teams: &[Thing],
        id: &str,
    ) -> Result<Collection, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("collection", id)?;
        let mut response = db
            .db
            .query("DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE")
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<CollectionRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::NotFound("collection not found".into()))
    }

    async fn add_song_to_collection(
        &self,
        write_teams: &[Thing],
        id: &str,
        song_link: SongLink,
    ) -> Result<(), AppError> {
        let db = self.inner();
        let _ = db
            .db
            .query(
                r#"UPDATE type::thing("collection", $id) SET songs = array::append(songs, $song) WHERE owner IN $teams;"#,
            )
            .bind(("id", id.to_owned()))
            .bind(("teams", write_teams.to_vec()))
            .bind(("song", SongLinkRecord::from(song_link)))
            .await?;

        Ok(())
    }
}
