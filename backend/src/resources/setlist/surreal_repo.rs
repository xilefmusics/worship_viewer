use actix_web::web::Data;
use async_trait::async_trait;
use surrealdb::sql::Thing;

use shared::api::ListQuery;
use shared::setlist::{CreateSetlist, Setlist};
use shared::song::LinkOwned as SongLinkOwned;

use crate::database::Database;
use crate::error::AppError;

use super::model::{SetlistRecord, SetlistSongsRecord, setlist_belongs_to, setlist_resource};
use super::repository::SetlistRepository;

#[derive(Clone)]
pub struct SurrealSetlistRepo {
    db: Data<Database>,
}

impl SurrealSetlistRepo {
    pub fn new(db: Data<Database>) -> Self {
        Self { db }
    }

    fn inner(&self) -> &Database {
        self.db.get_ref()
    }
}

#[async_trait]
impl SetlistRepository for SurrealSetlistRepo {
    async fn get_setlists(
        &self,
        read_teams: Vec<Thing>,
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

        let mut request = db.db.query(query).bind(("teams", read_teams));
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

    async fn get_setlist(&self, read_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError> {
        let db = self.inner();
        match db.db.select(setlist_resource(id)?).await? {
            Some(record) => {
                if setlist_belongs_to(&record, &read_teams) {
                    Ok(record.into_setlist())
                } else {
                    Err(AppError::NotFound("setlist not found".into()))
                }
            }
            None => Err(AppError::NotFound("setlist not found".into())),
        }
    }

    async fn get_setlist_songs(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError> {
        let db = self.inner();
        let resource = setlist_resource(id)?;
        let mut response = db
            .db
            .query("SELECT owner, songs FROM setlist WHERE id = $id FETCH songs.*.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<SetlistSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))?;

        if !record.belongs_to(&read_teams) {
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
        write_teams: Vec<Thing>,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let db = self.inner();
        let resource = setlist_resource(id)?;
        let owner_team = match db.db.select(resource.clone()).await? {
            Some(existing) => {
                if !setlist_belongs_to(&existing, &write_teams) {
                    return Err(AppError::NotFound("setlist not found".into()));
                }
                existing
                    .owner
                    .clone()
                    .ok_or_else(|| AppError::database("setlist missing owner"))?
            }
            None => {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        };

        let record_id = Thing::from(resource.clone());
        let record = SetlistRecord::from_payload(Some(record_id), Some(owner_team), setlist);

        if let Some(updated) = db
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(SetlistRecord::into_setlist)
        {
            return Ok(updated);
        }

        db.db
            .create(resource)
            .content(record)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to upsert setlist"))
    }

    async fn delete_setlist(&self, write_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError> {
        let db = self.inner();
        let resource = setlist_resource(id)?;
        if let Some(existing) = db.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, &write_teams) {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        } else {
            return Err(AppError::NotFound("setlist not found".into()));
        }

        db.db
            .delete(resource)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }
}
