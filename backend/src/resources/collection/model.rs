use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    api::ListQuery,
    collection::{Collection, CreateCollection},
    player::Player,
    song::{Link as SongLink, LinkOwned as SongLinkOwned, SimpleChord, Song},
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::song::{Model as SongDbModel, SongRecord, export, Format};
use crate::resources::team::{content_read_team_things, content_write_team_things};
use crate::resources::User;

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
        match self.db.select(collection_resource(id)?).await? {
            Some(record) if collection_belongs_to(&record, &read_teams) => {
                Ok(record.into_collection())
            }
            _ => Err(AppError::NotFound("collection not found".into())),
        }
    }

    async fn get_collection_songs(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError> {
        let resource = collection_resource(id)?;
        let mut response = self
            .db
            .query("SELECT owner, songs FROM collection WHERE id = $id FETCH songs.*.id")
            .bind(("id", Thing::from(resource.clone())))
            .await?;

        let record = response
            .take::<Option<CollectionSongsRecord>>(0)?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !record.belongs_to(&read_teams) {
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
        let resource = collection_resource(id)?;
        let existing = self
            .db
            .select(resource.clone())
            .await?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

        if !collection_belongs_to(&existing, &write_teams) {
            return Err(AppError::NotFound("collection not found".into()));
        }

        let record_id = Thing::from(resource.clone());
        let owner_team = existing
            .owner
            .clone()
            .ok_or_else(|| AppError::database("collection missing owner"))?;
        let record = CollectionRecord::from_payload(Some(record_id), Some(owner_team), collection);

        if let Some(updated) = self
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(CollectionRecord::into_collection)
        {
            return Ok(updated);
        }

        self.db
            .create(resource)
            .content(record)
            .await?
            .map(CollectionRecord::into_collection)
            .ok_or_else(|| AppError::database("failed to upsert collection"))
    }

    async fn delete_collection(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Collection, AppError> {
        let resource = collection_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !collection_belongs_to(&existing, &write_teams) {
                return Err(AppError::NotFound("collection not found".into()));
            }
        } else {
            return Err(AppError::NotFound("collection not found".into()));
        }

        self.db
            .delete(resource)
            .await?
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
        let liked_set = SongDbModel::get_liked_set(self, &user.id).await?;
        let read_teams = content_read_team_things(self, user).await?;
        let links = self.get_collection_songs(read_teams, id).await?;
        collection_player_from_links(liked_set, links)
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
        let liked_set = SongDbModel::get_liked_set(self, &user.id).await?;
        let read_teams = content_read_team_things(self, user).await?;
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

fn collection_player_from_links(
    liked_set: std::collections::HashSet<String>,
    links: Vec<SongLinkOwned>,
) -> Result<Player, AppError> {
    links
        .into_iter()
        .enumerate()
        .map(|(idx, link)| {
            Player::from(SongLinkOwned {
                liked: liked_set.contains(&link.song.id),
                song: link.song,
                nr: Some(link.nr.unwrap_or_else(|| (idx + 1).to_string())),
                key: link.key,
            })
        })
        .try_fold(Player::default(), |acc, player| Ok::<Player, AppError>(acc + player))
}

fn collection_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "collection" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid collection id"));
    }

    Ok(("collection".to_owned(), id.to_owned()))
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

fn blob_thing(blob_id: &str) -> Thing {
    if let Ok(thing) = blob_id.parse::<Thing>()
        && thing.tb == "blob"
    {
        return thing;
    }

    Thing::from(("blob".to_owned(), blob_id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SongLinkRecord {
    id: Thing,
    #[serde(default)]
    nr: Option<String>,
    #[serde(default)]
    key: Option<SimpleChord>,
}

impl From<SongLinkRecord> for SongLink {
    fn from(record: SongLinkRecord) -> Self {
        Self {
            id: record.id.id.to_string(),
            nr: record.nr,
            key: record.key,
        }
    }
}

impl From<SongLink> for SongLinkRecord {
    fn from(link: SongLink) -> Self {
        Self {
            id: song_thing(&link.id),
            nr: link.nr,
            key: link.key,
        }
    }
}

fn song_thing(song_id: &str) -> Thing {
    if let Ok(thing) = song_id.parse::<Thing>()
        && thing.tb == "song"
    {
        return thing;
    }

    Thing::from(("song".to_owned(), song_id.to_owned()))
}

fn collection_belongs_to(record: &CollectionRecord, teams: &[Thing]) -> bool {
    record
        .owner
        .as_ref()
        .map(|t| teams.contains(t))
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct CollectionSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl CollectionSongsRecord {
    fn belongs_to(&self, teams: &[Thing]) -> bool {
        self.owner
            .as_ref()
            .map(|t| teams.contains(t))
            .unwrap_or(false)
    }

    fn into_songs(self) -> Vec<SongLinkOwned> {
        self.songs
            .into_iter()
            .map(|record| record.into_song_link_owned())
            .collect()
    }
}

#[derive(Deserialize)]
struct FetchedSongRecord {
    #[serde(rename = "id")]
    song: SongRecord,
    nr: Option<String>,
    key: Option<SimpleChord>,
    #[serde(default)]
    liked: bool,
}

impl FetchedSongRecord {
    fn into_song_link_owned(self) -> SongLinkOwned {
        SongLinkOwned {
            song: self.song.into_song(),
            nr: self.nr,
            key: self.key,
            liked: self.liked,
        }
    }
}
