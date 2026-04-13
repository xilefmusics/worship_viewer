use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use shared::{
    api::ListQuery,
    player::Player,
    setlist::{CreateSetlist, Setlist},
    song::{Link as SongLink, LinkOwned as SongLinkOwned, SimpleChord, Song},
};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::song::{Model as SongDbModel, SongRecord, export, Format};
use crate::resources::team::{content_read_team_things, content_write_team_things};
use crate::resources::User;

pub trait Model {
    async fn get_setlists(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError>;
    async fn get_setlist(&self, read_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError>;
    async fn get_setlist_songs(
        &self,
        read_teams: Vec<Thing>,
        id: &str,
    ) -> Result<Vec<SongLinkOwned>, AppError>;
    async fn create_setlist(
        &self,
        owner: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn update_setlist(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError>;
    async fn delete_setlist(&self, write_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError>;
}

impl Model for Database {
    async fn get_setlists(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError> {
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
            .take::<Vec<SetlistRecord>>(0)?
            .into_iter()
            .map(SetlistRecord::into_setlist)
            .collect())
    }

    async fn get_setlist(&self, read_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError> {
        match self.db.select(setlist_resource(id)?).await? {
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
        let resource = setlist_resource(id)?;
        let mut response = self
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
        let owner_team = self.personal_team_thing_for_user(owner).await?;
        self.db
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
        let resource = setlist_resource(id)?;
        let owner_team = match self.db.select(resource.clone()).await? {
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

        if let Some(updated) = self
            .db
            .update(resource.clone())
            .content(record.clone())
            .await?
            .map(SetlistRecord::into_setlist)
        {
            return Ok(updated);
        }

        self.db
            .create(resource)
            .content(record)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::database("failed to upsert setlist"))
    }

    async fn delete_setlist(&self, write_teams: Vec<Thing>, id: &str) -> Result<Setlist, AppError> {
        let resource = setlist_resource(id)?;
        if let Some(existing) = self.db.select(resource.clone()).await? {
            if !setlist_belongs_to(&existing, &write_teams) {
                return Err(AppError::NotFound("setlist not found".into()));
            }
        } else {
            return Err(AppError::NotFound("setlist not found".into()));
        }

        self.db
            .delete(resource)
            .await?
            .map(SetlistRecord::into_setlist)
            .ok_or_else(|| AppError::NotFound("setlist not found".into()))
    }
}

impl Database {
    pub async fn list_setlists_for_user(
        &self,
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Setlist>, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_setlists(read_teams, pagination).await
    }

    pub async fn get_setlist_for_user(&self, user: &User, id: &str) -> Result<Setlist, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_setlist(read_teams, id).await
    }

    pub async fn setlist_player_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Player, AppError> {
        let liked_set = SongDbModel::get_liked_set(self, &user.id).await?;
        let read_teams = content_read_team_things(self, user).await?;
        let links = self.get_setlist_songs(read_teams, id).await?;
        player_from_song_links(liked_set, links)
    }

    pub async fn export_setlist_for_user(
        &self,
        user: &User,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let songs: Vec<Song> = self
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
        let liked_set = SongDbModel::get_liked_set(self, &user.id).await?;
        let read_teams = content_read_team_things(self, user).await?;
        Ok(self
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
        self.create_setlist(&user.id, setlist).await
    }

    pub async fn update_setlist_for_user(
        &self,
        user: &User,
        id: &str,
        setlist: CreateSetlist,
    ) -> Result<Setlist, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.update_setlist(write_teams, id, setlist).await
    }

    pub async fn delete_setlist_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<Setlist, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.delete_setlist(write_teams, id).await
    }
}

fn player_from_song_links(
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

fn setlist_resource(id: &str) -> Result<(String, String), AppError> {
    if let Ok(thing) = id.parse::<Thing>() {
        if thing.tb == "setlist" {
            return Ok((thing.tb, thing.id.to_string()));
        }
        return Err(AppError::invalid_request("invalid setlist id"));
    }

    Ok(("setlist".to_owned(), id.to_owned()))
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SetlistRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    title: String,
    #[serde(default)]
    songs: Vec<SongLinkRecord>,
}

impl SetlistRecord {
    fn into_setlist(self) -> Setlist {
        Setlist {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            title: self.title,
            songs: self.songs.into_iter().map(Into::into).collect(),
        }
    }

    fn from_payload(id: Option<Thing>, owner: Option<Thing>, setlist: CreateSetlist) -> Self {
        Self {
            id,
            owner,
            title: setlist.title,
            songs: setlist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
struct SetlistSongsRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    #[serde(default)]
    songs: Vec<FetchedSongRecord>,
}

impl SetlistSongsRecord {
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
    #[serde(default)]
    nr: Option<String>,
    #[serde(default)]
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

fn setlist_belongs_to(record: &SetlistRecord, teams: &[Thing]) -> bool {
    record
        .owner
        .as_ref()
        .map(|t| teams.contains(t))
        .unwrap_or(false)
}

fn song_thing(id: &str) -> Thing {
    if let Ok(thing) = id.parse::<Thing>()
        && thing.tb == "song"
    {
        return thing;
    }

    Thing::from(("song".to_owned(), id.to_owned()))
}
