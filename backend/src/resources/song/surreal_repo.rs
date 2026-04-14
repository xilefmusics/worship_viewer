use std::collections::HashSet;

use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::song::{CreateSong, Song};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::{belongs_to, blob_thing, resource_id};

use super::model::{LikeRecord, SongRecord, search_content_from_song_data};
use super::repository::SongRepository;

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

#[derive(Clone)]
pub struct SurrealSongRepo {
    db: Arc<Database>,
}

impl SurrealSongRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl SongRepository for SurrealSongRepo {
    async fn get_songs(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError> {
        let db = self.inner();
        let q_nonempty = pagination.q.as_ref().is_some_and(|q| !q.trim().is_empty());
        let mut query = if q_nonempty {
            String::from(
                "SELECT *, ((search::score(0) ?? 0) * 100 + (search::score(1) ?? 0) * 10 + (search::score(2) ?? 0) * 1) AS score FROM song WHERE owner IN $teams",
            )
        } else {
            String::from("SELECT * FROM song WHERE owner IN $teams")
        };
        if q_nonempty {
            query.push_str(
                " AND (data.titles @0@ $q OR data.artists @1@ $q OR search_content @2@ $q) ORDER BY score DESC",
            );
        }
        if pagination.to_offset_limit().is_some() {
            query.push_str(" LIMIT $limit START $start");
        }

        let mut request = db.db.query(query).bind(("teams", read_teams.to_vec()));
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
            .take::<Vec<SongRecord>>(0)?
            .into_iter()
            .map(SongRecord::into_song)
            .collect())
    }

    async fn get_song(&self, read_teams: &[Thing], id: &str) -> Result<Song, AppError> {
        let db = self.inner();
        let record: Option<SongRecord> = db.db.select(resource_id("song", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, &read_teams) => Ok(r.into_song()),
            _ => Err(AppError::NotFound("song not found".into())),
        }
    }

    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError> {
        let db = self.inner();
        let owner_team = db.personal_team_thing_for_user(owner).await?;
        db.db
            .create("song")
            .content(SongRecord::from_payload(None, Some(owner_team), song))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to create song"))
    }

    async fn update_song(
        &self,
        write_teams: &[Thing],
        actor_user_id: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let db = self.inner();
        let resource = resource_id("song", id)?;
        let (tb, sid) = resource.clone();
        let search_content = search_content_from_song_data(&song.data);
        let blobs: Vec<Thing> = song.blobs.iter().map(|b| blob_thing(b)).collect();

        let mut response = db
            .db
            .query(
                "UPDATE type::thing($tb, $sid) SET not_a_song = $not_a_song, blobs = $blobs, \
                 data = $data, search_content = $search_content WHERE owner IN $teams RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("not_a_song", song.not_a_song))
            .bind(("blobs", blobs))
            .bind(("data", song.data.clone()))
            .bind(("search_content", search_content))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<SongRecord> = response.take(0)?;
        if let Some(updated) = rows.into_iter().next() {
            return Ok(updated.into_song());
        }

        let existing: Option<SongRecord> = db.db.select(resource.clone()).await?;
        if existing.is_some() {
            return Err(AppError::NotFound("song not found".into()));
        }

        let personal = db.personal_team_thing_for_user(actor_user_id).await?;
        if !write_teams.contains(&personal) {
            return Err(AppError::NotFound("song not found".into()));
        }
        let record_id = Thing::from(resource.clone());
        let record = SongRecord::from_payload(Some(record_id), Some(personal), song);
        db.db
            .create(resource)
            .content(record)
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to upsert song"))
    }

    async fn delete_song(&self, write_teams: &[Thing], id: &str) -> Result<Song, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("song", id)?;
        let mut response = db
            .db
            .query("DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE")
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<SongRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::NotFound("song not found".into()))
    }

    async fn get_song_like(
        &self,
        read_teams: &[Thing],
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError> {
        let db = self.inner();
        let resource = resource_id("song", id)?;
        let existing: SongRecord = db
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !belongs_to(&existing.owner, &read_teams) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = db
            .db
            .query("SELECT * FROM like WHERE owner = $owner AND song = $song LIMIT 1")
            .bind(("owner", owner))
            .bind(("song", song))
            .await?;

        let likes: Vec<LikeRecord> = response.take(0)?;
        Ok(!likes.is_empty())
    }

    async fn set_song_like(
        &self,
        read_teams: &[Thing],
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError> {
        let db = self.inner();
        let resource = resource_id("song", id)?;
        let existing: SongRecord = db
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !belongs_to(&existing.owner, &read_teams) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = db
            .db
            .query("SELECT * FROM like WHERE owner = $owner AND song = $song LIMIT 1")
            .bind(("owner", owner.clone()))
            .bind(("song", song.clone()))
            .await?;

        let mut likes: Vec<LikeRecord> = response.take(0)?;
        let existing_like = likes.pop();

        if liked {
            if existing_like.is_none() {
                let _: Option<LikeRecord> = db
                    .db
                    .create("like")
                    .content(LikeRecord::new(owner, song))
                    .await?;
            }
            Ok(true)
        } else if let Some(record) = existing_like.and_then(|like| like.id) {
            let resource = (record.tb.clone(), record.id.to_string());
            let _: Option<LikeRecord> = db.db.delete(resource).await?;
            Ok(false)
        } else {
            Ok(false)
        }
    }

    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        let db = self.inner();
        let mut response = db
            .db
            .query("SELECT * FROM like WHERE owner = $owner")
            .bind(("owner", owner_thing(user_id)))
            .await?;

        let likes: Vec<LikeRecord> = response.take(0)?;
        Ok(likes
            .into_iter()
            .map(|like| like.song.id.to_string())
            .collect())
    }
}
