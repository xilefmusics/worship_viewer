use std::collections::HashSet;

use actix_web::HttpResponse;
use chordlib::types::Song as SongData;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

use shared::api::ListQuery;
use shared::like::LikeStatus;
use shared::player::Player;
use shared::song::{
    CreateSong, Link as SongLink, LinkOwned as SongLinkOwned, Song, SongUserSpecificAddons,
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::collection::{CreateCollection, Model as CollectionDbModel};
use crate::resources::common::{belongs_to, blob_thing, resource_id};
use crate::resources::team::{content_read_team_things, content_write_team_things};
use crate::resources::user::Model as UserDbModel;

use super::{Format, export};

pub trait Model {
    async fn get_songs(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError>;
    async fn get_song(&self, read_teams: Vec<Thing>, id: &str) -> Result<Song, AppError>;
    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError>;
    async fn update_song(
        &self,
        write_teams: Vec<Thing>,
        actor_user_id: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError>;
    async fn delete_song(&self, write_teams: Vec<Thing>, id: &str) -> Result<Song, AppError>;
    async fn get_song_like(
        &self,
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError>;
    async fn set_song_like(
        &self,
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError>;
    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError>;
}

impl Model for Database {
    async fn get_songs(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError> {
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
            .take::<Vec<SongRecord>>(0)?
            .into_iter()
            .map(SongRecord::into_song)
            .collect())
    }

    async fn get_song(&self, read_teams: Vec<Thing>, id: &str) -> Result<Song, AppError> {
        let record: Option<SongRecord> = self.db.select(resource_id("song", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, &read_teams) => Ok(r.into_song()),
            _ => Err(AppError::NotFound("song not found".into())),
        }
    }

    async fn create_song(&self, owner: &str, song: CreateSong) -> Result<Song, AppError> {
        let owner_team = self.personal_team_thing_for_user(owner).await?;
        self.db
            .create("song")
            .content(SongRecord::from_payload(None, Some(owner_team), song))
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to create song"))
    }

    async fn update_song(
        &self,
        write_teams: Vec<Thing>,
        actor_user_id: &str,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let resource = resource_id("song", id)?;
        let (tb, sid) = resource.clone();
        let search_content = search_content_from_song_data(&song.data);
        let blobs: Vec<Thing> = song.blobs.iter().map(|b| blob_thing(b)).collect();

        let mut response = self
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
            .bind(("teams", write_teams.clone()))
            .await?;

        let rows: Vec<SongRecord> = response.take(0)?;
        if let Some(updated) = rows.into_iter().next() {
            return Ok(updated.into_song());
        }

        let existing: Option<SongRecord> = self.db.select(resource.clone()).await?;
        if existing.is_some() {
            return Err(AppError::NotFound("song not found".into()));
        }

        let personal = self.personal_team_thing_for_user(actor_user_id).await?;
        if !write_teams.contains(&personal) {
            return Err(AppError::NotFound("song not found".into()));
        }
        let record_id = Thing::from(resource.clone());
        let record = SongRecord::from_payload(Some(record_id), Some(personal), song);
        self.db
            .create(resource)
            .content(record)
            .await?
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::database("failed to upsert song"))
    }

    async fn delete_song(&self, write_teams: Vec<Thing>, id: &str) -> Result<Song, AppError> {
        let (tb, sid) = resource_id("song", id)?;
        let mut response = self
            .db
            .query(
                "DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams))
            .await?;

        let rows: Vec<SongRecord> = response.take(0)?;
        rows
            .into_iter()
            .next()
            .map(SongRecord::into_song)
            .ok_or_else(|| AppError::NotFound("song not found".into()))
    }

    async fn get_song_like(
        &self,
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
    ) -> Result<bool, AppError> {
        let resource = resource_id("song", id)?;
        let existing: SongRecord = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !belongs_to(&existing.owner, &read_teams) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = self
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
        read_teams: Vec<Thing>,
        user_id: &str,
        id: &str,
        liked: bool,
    ) -> Result<bool, AppError> {
        let resource = resource_id("song", id)?;
        let existing: SongRecord = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("song not found".into()))?;

        if !belongs_to(&existing.owner, &read_teams) {
            return Err(AppError::NotFound("song not found".into()));
        }

        let owner = owner_thing(user_id);
        let song = Thing::from(resource);

        let mut response = self
            .db
            .query("SELECT * FROM like WHERE owner = $owner AND song = $song LIMIT 1")
            .bind(("owner", owner.clone()))
            .bind(("song", song.clone()))
            .await?;

        let mut likes: Vec<LikeRecord> = response.take(0)?;
        let existing_like = likes.pop();

        if liked {
            if existing_like.is_none() {
                let _: Option<LikeRecord> = self
                    .db
                    .create("like")
                    .content(LikeRecord::new(owner, song))
                    .await?;
            }
            Ok(true)
        } else if let Some(record) = existing_like.and_then(|like| like.id) {
            let resource = (record.tb.clone(), record.id.to_string());
            let _: Option<LikeRecord> = self.db.delete(resource).await?;
            Ok(false)
        } else {
            Ok(false)
        }
    }
    async fn get_liked_set(&self, user_id: &str) -> Result<HashSet<String>, AppError> {
        let mut response = self
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

impl Database {
    pub async fn list_songs_for_user(
        &self,
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Song>, AppError> {
        let (liked_set, read_teams) = tokio::try_join!(
            self.get_liked_set(&user.id),
            content_read_team_things(self, user)
        )?;
        Ok(self
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
            self.get_liked_set(&user.id),
            content_read_team_things(self, user)
        )?;
        let mut song = self.get_song(read_teams, id).await?;
        song.user_specific_addons.liked = liked_set.contains(&song.id);
        Ok(song)
    }

    pub async fn song_player_for_user(&self, user: &User, id: &str) -> Result<Player, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        Ok(Player::from(SongLinkOwned {
            song: self.get_song(read_teams.clone(), id).await?,
            nr: None,
            key: None,
            liked: self.get_song_like(read_teams, &user.id, id).await?,
        }))
    }

    pub async fn export_song_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let song = self.get_song(read_teams, id).await?;
        export(vec![song], format).await
    }

    pub async fn create_song_for_user(
        &self,
        user: &User,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let created = self.create_song(&user.id, song).await?;

        if let Some(collection_id) = user.default_collection.as_ref() {
            let write_teams = content_write_team_things(self, user).await?;
            CollectionDbModel::add_song_to_collection(
                self,
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
            let collection = CollectionDbModel::create_collection(
                self,
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
            UserDbModel::set_default_collection_to_user(self, &user.id, &collection.id).await?;
        }

        Ok(created)
    }

    pub async fn update_song_for_user(
        &self,
        user: &User,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.update_song(write_teams, &user.id, id, song).await
    }

    pub async fn delete_song_for_user(&self, user: &User, id: &str) -> Result<Song, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.delete_song(write_teams, id).await
    }

    pub async fn song_like_status_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let liked = self.get_song_like(read_teams, &user.id, id).await?;
        Ok(LikeStatus { liked })
    }

    pub async fn set_song_like_status_for_user(
        &self,
        user: &User,
        id: &str,
        liked: bool,
    ) -> Result<LikeStatus, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let liked = self.set_song_like(read_teams, &user.id, id, liked).await?;
        Ok(LikeStatus { liked })
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SongRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    not_a_song: bool,
    #[serde(default)]
    blobs: Vec<Thing>,
    data: SongData,
    #[serde(default)]
    search_content: String,
}

impl SongRecord {
    pub fn into_song(self) -> Song {
        Song {
            id: self.id.map(id_from_thing).unwrap_or_default(),
            owner: self.owner.map(id_from_thing).unwrap_or_default(),
            not_a_song: self.not_a_song,
            blobs: self.blobs.into_iter().map(id_from_thing).collect(),
            data: self.data,
            user_specific_addons: SongUserSpecificAddons::default(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, song: CreateSong) -> Self {
        let search_content = search_content_from_song_data(&song.data);
        Self {
            id,
            owner,
            not_a_song: song.not_a_song,
            blobs: song
                .blobs
                .into_iter()
                .map(|blob_id| blob_thing(&blob_id))
                .collect(),
            data: song.data,
            search_content,
        }
    }
}

fn search_content_from_song_data(data: &SongData) -> String {
    let mut pieces: Vec<String> = Vec::new();
    for section in &data.sections {
        for line in &section.lines {
            for part in &line.parts {
                for text in &part.languages {
                    if !text.is_empty() {
                        pieces.push(text.clone());
                    }
                }
            }
        }
    }
    pieces.join(" ")
}

fn owner_thing(user_id: &str) -> Thing {
    Thing::from(("user".to_owned(), user_id.to_owned()))
}

fn id_from_thing(thing: Thing) -> String {
    id_to_plain_string(thing.id)
}

fn id_to_plain_string(id: Id) -> String {
    match id {
        Id::String(value) => value,
        Id::Number(number) => format!("{number}"),
        Id::Uuid(uuid) => uuid.to_string(),
        _ => id.to_string(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LikeRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    owner: Thing,
    song: Thing,
}

impl LikeRecord {
    fn new(owner: Thing, song: Thing) -> Self {
        Self {
            id: None,
            owner,
            song,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_record_into_song_maps_string_ids() {
        let record = SongRecord {
            id: Some(Thing::from(("song".to_owned(), "s1".to_owned()))),
            owner: Some(Thing::from(("team".to_owned(), "t9".to_owned()))),
            not_a_song: true,
            blobs: vec![Thing::from(("blob".to_owned(), "b1".to_owned()))],
            data: SongData::default(),
            search_content: String::new(),
        };
        let song = record.into_song();
        assert_eq!(song.id, "s1");
        assert_eq!(song.owner, "t9");
        assert!(song.not_a_song);
        assert_eq!(song.blobs, vec!["b1".to_string()]);
    }

    #[test]
    fn song_record_from_payload_sets_search_content_and_blob_things() {
        let data: SongData = serde_json::from_str(
            r#"{
                "titles": ["T"],
                "sections": [{
                    "title": "V",
                    "lines": [{
                        "parts": [{
                            "languages": ["Hello", "world"],
                            "comment": false
                        }]
                    }]
                }]
            }"#,
        )
        .expect("song data json");
        let create = CreateSong {
            not_a_song: false,
            blobs: vec!["blob:bb".into(), "rawblob".into()],
            data,
        };
        let record = SongRecord::from_payload(None, None, create);
        assert_eq!(record.blobs.len(), 2);
        assert_eq!(record.blobs[0].tb, "blob");
        assert_eq!(record.blobs[1].tb, "blob");
        assert_eq!(record.search_content, "Hello world");
    }

    #[test]
    fn search_content_from_song_data_empty() {
        assert_eq!(search_content_from_song_data(&SongData::default()), "");
    }

    #[test]
    fn search_content_from_song_data_joins_non_empty_languages() {
        let data: SongData = serde_json::from_str(
            r#"{
                "titles": ["T"],
                "sections": [{
                    "title": "V",
                    "lines": [{
                        "parts": [{
                            "languages": ["one", "two"],
                            "comment": false
                        }]
                    }]
                }]
            }"#,
        )
        .expect("song data json");
        assert_eq!(search_content_from_song_data(&data), "one two");
    }

    #[tokio::test]
    async fn blc_song_crud_search_likes() {
        use shared::api::ListQuery;
        use shared::team::TeamRole;

        use crate::test_helpers::{
            configure_personal_team_members, create_song_with_title, create_user,
            personal_team_id, test_db,
        };

        let db = test_db().await.expect("db");
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

        let list = db
            .list_songs_for_user(&owner, ListQuery::default())
            .await
            .expect("list");
        assert!(list.len() >= 2);

        let q = db
            .list_songs_for_user(&owner, ListQuery::new().with_q("Alpha"))
            .await
            .expect("search");
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].id, s1.id);

        db.get_song_for_user(&owner, &s1.id).await.expect("get");

        db.get_song_for_user(&other, &s1.id)
            .await
            .expect("guest read");

        let bad = db.get_song_for_user(&owner, "setlist:not-a-song").await;
        assert!(bad.is_err(), "wrong table id should not resolve: {bad:?}");

        db.set_song_like_status_for_user(&owner, &s1.id, true)
            .await
            .expect("like");
        let st = db
            .song_like_status_for_user(&owner, &s1.id)
            .await
            .expect("like status");
        assert!(st.liked);

        db.delete_song_for_user(&owner, &s1.id).await.expect("del");
    }

    #[tokio::test]
    async fn blc_song_delete_after_setlist_link() {
        use crate::test_helpers::{
            create_song_with_title, create_user, setlist_service, setlist_with_songs, test_db,
        };

        let db = test_db().await.expect("db");
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
        db.delete_song_for_user(&u, &song.id)
            .await
            .expect("del song");
        let g = setlist_service(&db)
            .get_setlist_for_user(&u, &sl.id)
            .await
            .expect("get setlist");
        assert!(g.songs.is_empty());
    }
}
