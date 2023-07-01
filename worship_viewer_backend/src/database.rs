use super::error::AppError;
use super::types::{
    Blob, Collection, CollectionDatabase, Group, Song, SongDatabase, TitleAndSongAndBlobs, User,
    UserGroupsFetched, UserGroupsId,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SimpleValue<T> {
    pub value: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Response<T> {
    pub result: T,
    pub status: String,
}

impl<T> Response<T> {
    pub fn validate(self) -> Result<T, AppError> {
        match self.status.as_str() {
            "OK" => Ok(self.result),
            status => Err(AppError::Database(format!(
                "The status of the response is {}",
                status
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelateResponse {
    pub id: String,
    #[serde(rename = "in")]
    pub in_link: String,
    #[serde(rename = "out")]
    pub out_link: String,
}

pub struct Database {
    url: String,
    auth: String,
    namespace: String,
    database: String,
}

impl Database {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
    ) -> Self {
        let url = format!("http://{}:{}/sql", host, port);
        let auth = format!(
            "Basic {}",
            base64::encode(format!("{}:{}", username, password))
        );
        let namespace = namespace.into();
        let database = database.into();

        Self {
            url,
            auth,
            namespace,
            database,
        }
    }

    async fn query_string(&self, query: String) -> Result<String, AppError> {
        Ok(reqwest::Client::new()
            .post(&self.url)
            .header("Accept", "application/json")
            .header("NS", &self.namespace)
            .header("DB", &self.database)
            .header("Authorization", &self.auth)
            .body(query)
            .send()
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .text()
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?)
    }

    pub async fn query_vec<T: DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
        query: String,
    ) -> Result<T, AppError> {
        let response_string = dbg!(self.query_string(query.clone()).await?);

        let response_objects = serde_json::from_str::<Vec<Response<T>>>(&response_string)
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .into_iter()
            .map(|response| response.validate())
            .collect::<Result<Vec<T>, AppError>>()?;

        if response_objects.len() == 0 {
            return Err(AppError::Database(format!(
                "got no response for query ({})",
                query
            )));
        }

        Ok(response_objects[0].clone())
    }

    pub async fn query_one<T: DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
        query: String,
    ) -> Result<T, AppError> {
        let vec = self.query_vec::<Vec<T>>(query).await?;
        if vec.len() != 1 {
            return Err(AppError::NotFound("item not found".into()));
        }
        Ok(vec[0].clone())
    }

    pub async fn query_check<T: DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
        query: String,
    ) -> Result<bool, AppError> {
        let vec = self.query_vec::<Vec<T>>(query).await?;
        Ok(vec.len() > 0)
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>, AppError> {
        self.query_vec("SELECT * FROM group;".into()).await
    }

    pub async fn get_group(&self, id: &str) -> Result<Group, AppError> {
        self.query_one(format!("SELECT * FROM group WHERE id = {};", id))
            .await
    }

    pub async fn add_groups(&self, blobs: &Vec<Group>) -> Result<Vec<Group>, AppError> {
        self.query_vec(format!(
            "INSERT INTO group {};",
            serde_json::to_string(blobs).map_err(|err| AppError::Database(format!("{}", err)))?
        ))
        .await
    }

    pub async fn get_users(&self) -> Result<Vec<UserGroupsFetched>, AppError> {
        self.query_vec("SELECT *, ->member_of->group as groups FROM user FETCH groups;".into())
            .await
    }

    pub async fn get_user(&self, id: &str) -> Result<UserGroupsFetched, AppError> {
        self.query_one(format!(
            "SELECT *, ->member_of->group as groups FROM user WHERE id = {} FETCH groups;",
            id
        ))
        .await
    }

    pub async fn add_users(&self, users: &Vec<UserGroupsId>) -> Result<Vec<User>, AppError> {
        let relate_query = users
            .iter()
            .map(|user| {
                user.groups.iter().map(|group| {
                    format!(
                        "RELATE {}->member_of->{};",
                        &user.id.clone().unwrap_or(String::new()),
                        group
                    )
                })
            })
            .flatten()
            .collect::<String>();

        let users = users
            .into_iter()
            .map(|user| user.drop_groups())
            .collect::<Vec<User>>();

        let created_users: Vec<User> = self
            .query_vec(format!(
                "INSERT INTO user {};",
                serde_json::to_string(&dbg!(users))
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?;

        let _: Vec<RelateResponse> = self.query_vec(dbg!(relate_query)).await?;

        Ok(created_users)
    }

    pub async fn get_blobs(&self, username: &str) -> Result<Vec<Blob>, AppError> {
        self.query_vec(format!("SELECT * FROM blob WHERE (->owned_by->group<-member_of<-user).name contains \"{}\";", username).into()).await
    }

    pub async fn get_blob(&self, username: &str, id: &str) -> Result<Blob, AppError> {
        self.query_one(format!("SELECT * FROM blob WHERE id = {} AND (->owned_by->group<-member_of<-user).name contains \"{}\";", id, username).into()).await
    }

    pub async fn add_blobs(&self, blobs: &Vec<Blob>, group: &str) -> Result<Vec<Blob>, AppError> {
        let created_blobs: Vec<Blob> = self
            .query_vec(format!(
                "INSERT INTO blob {};",
                serde_json::to_string(blobs)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?;

        let relate_query = created_blobs
            .iter()
            .map(|blob| {
                format!(
                    "RELATE {}->owned_by->{};",
                    &blob.id.clone().unwrap_or(String::new()),
                    group
                )
            })
            .collect::<String>();

        let _: Vec<RelateResponse> = self.query_vec(relate_query).await?;

        Ok(blobs.clone())
    }

    pub async fn get_songs(&self, username: &str) -> Result<Vec<Song>, AppError> {
        self.query_vec(format!("SELECT *, (SELECT * FROM $parent->has_blob ORDER BY idx).out as blobs FROM song WHERE (->owned_by->group<-member_of<-user).name contains \"{}\";", username).into()).await
    }

    pub async fn get_song(&self, username: &str, id: &str) -> Result<Song, AppError> {
        self.query_one(format!("SELECT *, (SELECT * FROM $parent->has_blob ORDER BY idx).out as blobs FROM song WHERE (->owned_by->group<-member_of<-user).name contains \"{}\" AND id = {};", username, id).into()).await
    }

    pub async fn add_songs(
        &self,
        songs: &Vec<Song>,
        group: &str,
    ) -> Result<Vec<SongDatabase>, AppError> {
        let database_songs = songs
            .iter()
            .map(|song| song.clone().drop_blobs())
            .collect::<Vec<SongDatabase>>();

        let created_songs: Vec<SongDatabase> = self
            .query_vec(format!(
                "INSERT INTO song {};",
                serde_json::to_string(&database_songs)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?;

        let relate_group_query = created_songs
            .iter()
            .map(|song| {
                format!(
                    "RELATE {}->owned_by->{};",
                    &song.id.clone().unwrap_or(String::new()),
                    group
                )
            })
            .collect::<String>();

        let _: Vec<RelateResponse> = self.query_vec(relate_group_query).await?;

        let relate_blobs_query = songs
            .into_iter()
            .map(|song| {
                song.blobs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(idx, blob)| {
                        format!(
                            "RELATE {}->has_blob->{} SET idx = {};",
                            song.id.clone().unwrap_or(String::new()),
                            blob,
                            idx
                        )
                    })
                    .collect::<String>()
            })
            .collect::<String>();

        let _: Vec<RelateResponse> = self.query_vec(relate_blobs_query).await?;

        Ok(created_songs.clone())
    }

    pub async fn get_collections(&self, username: &str) -> Result<Vec<Collection>, AppError> {
        self.query_vec(format!("SELECT *, (SELECT * FROM $parent->has_song ORDER BY idx).out as songs FROM collection WHERE (->owned_by->group<-member_of<-user).name contains \"{}\";", username).into()).await
    }

    pub async fn get_collection(&self, username: &str, id: &str) -> Result<Collection, AppError> {
        self.query_one(format!("SELECT *, (SELECT * FROM $parent->has_song ORDER BY idx).out as songs FROM collection WHERE (->owned_by->group<-member_of<-user).name contains \"{}\" AND id = {};", username, id).into()).await
    }

    pub async fn get_title_and_song_and_blobs_for_collection(
        &self,
        username: &str,
        id: &str,
    ) -> Result<Vec<TitleAndSongAndBlobs>, AppError> {
        self.query_vec(format!("SELECT title, song, blobs FROM (SELECT out.title as title, out as song, idx, (SELECT * FROM has_blob WHERE in = $parent.out ORDER BY idx).out as blobs FROM has_song WHERE in = {} ORDER BY idx);", id)).await
    }

    pub async fn add_collections(
        &self,
        songs: &Vec<Collection>,
        group: &str,
    ) -> Result<Vec<CollectionDatabase>, AppError> {
        let database_collections = songs
            .iter()
            .map(|song| CollectionDatabase::from_collection(song.clone()))
            .collect::<Vec<CollectionDatabase>>();

        let created_collections: Vec<CollectionDatabase> = self
            .query_vec(format!(
                "INSERT INTO collection {};",
                serde_json::to_string(&database_collections)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?;

        let relate_group_query = created_collections
            .iter()
            .map(|collection| {
                format!(
                    "RELATE {}->owned_by->{};",
                    &collection.id.clone().unwrap_or(String::new()),
                    group
                )
            })
            .collect::<String>();

        let _: Vec<RelateResponse> = self.query_vec(relate_group_query).await?;

        let relate_songs_query = songs
            .into_iter()
            .map(|collection| {
                collection
                    .songs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(idx, song)| {
                        format!(
                            "RELATE {}->has_song->{} SET idx = {};",
                            collection.id.clone().unwrap_or(String::new()),
                            song,
                            idx
                        )
                    })
                    .collect::<String>()
            })
            .collect::<String>();

        let _: Vec<RelateResponse> = self.query_vec(relate_songs_query).await?;

        Ok(created_collections.clone())
    }

    pub async fn check_blob(&self, id: &str, username: &str) -> Result<bool, AppError> {
        self.query_check::<SimpleValue<String>>(format!("SELECT id as value FROM blob WHERE id = {} AND (->owned_by->group<-member_of<-user).name contains \"{}\";",id, username)).await
    }

    pub async fn check_user(&self, username: &str) -> Result<bool, AppError> {
        self.query_check::<SimpleValue<String>>(format!(
            "SELECT name as value FROM user WHERE name = \"{}\";",
            username
        ))
        .await
    }

    pub async fn check_user_admin(&self, username: &str) -> Result<bool, AppError> {
        self.query_check::<SimpleValue<String>>(format!("SELECT name as value FROM user WHERE name = \"{}\" AND (->member_of->group).name CONTAINS \"admin\";",username)).await
    }
}
