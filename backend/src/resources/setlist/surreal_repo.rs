use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::setlist::{CreateSetlist, Setlist};
use shared::song::LinkOwned as SongLinkOwned;

use crate::database::Database;
use crate::error::AppError;

use crate::resources::common::{SongLinkRecord, belongs_to, resource_id};

use super::model::{SetlistRecord, SetlistSongsRecord};
use super::repository::SetlistRepository;

#[derive(Clone)]
pub struct SurrealSetlistRepo {
    db: Arc<Database>,
}

impl SurrealSetlistRepo {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl SetlistRepository for SurrealSetlistRepo {
    async fn get_setlists(
        &self,
        read_teams: &[Thing],
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError> {
        let db = self.inner();
        let q_nonempty = pagination.q.as_ref().is_some_and(|q| !q.trim().is_empty());
        let mut query = if q_nonempty {
            String::from(
                "SELECT *, (search::score(0) ?? 0) AS score FROM setlist WHERE owner IN $teams",
            )
        } else {
            String::from("SELECT * FROM setlist WHERE owner IN $teams")
        };
        if q_nonempty {
            query.push_str(" AND title @0@ $q ORDER BY score DESC");
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
            .take::<Vec<SetlistRecord>>(0)?
            .into_iter()
            .map(SetlistRecord::into_setlist)
            .collect())
    }

    async fn get_setlist(&self, read_teams: &[Thing], id: &str) -> Result<Setlist, AppError> {
        let db = self.inner();
        let record: Option<SetlistRecord> = db.db.select(resource_id("setlist", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, read_teams) => Ok(r.into_setlist()),
            _ => Err(AppError::NotFound("setlist not found".into())),
        }
    }

    async fn get_setlist_songs(
        &self,
        read_teams: &[Thing],
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError> {
        let db = self.inner();
        let resource = resource_id("setlist", id)?;
        let mut response = db
            .db
            .query("SELECT owner, songs FROM setlist WHERE id = $id FETCH songs.*.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<SetlistSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))?;

        if !record.belongs_to(read_teams) {
            return Err(AppError::NotFound("setlist not found".into()));
        }

        Ok(record.into_songs())
    }

    async fn create_setlist(
        &self,
        owner: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let db = self.inner();
        let owner_team = db.personal_team_thing_for_user(owner).await?;
        db.db
            .create("setlist")
            .content(SetlistRecord::from_payload(None, Some(owner_team), setlist))
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to create setlist"))
    }

    async fn update_setlist(
        &self,
        write_teams: &[Thing],
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("setlist", id)?;
        let songs: Vec<SongLinkRecord> = setlist.songs.into_iter().map(Into::into).collect();
        let title = setlist.title;

        let mut response = db
            .db
            .query(
                "UPDATE type::thing($tb, $sid) SET title = $title, songs = $songs \
                 WHERE owner IN $teams RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("title", title))
            .bind(("songs", songs))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<SetlistRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }

    async fn delete_setlist(&self, write_teams: &[Thing], id: &str) -> Result<Setlist, AppError> {
        let db = self.inner();
        let (tb, sid) = resource_id("setlist", id)?;
        let mut response = db
            .db
            .query("DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE")
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams.to_vec()))
            .await?;

        let rows: Vec<SetlistRecord> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }
}
