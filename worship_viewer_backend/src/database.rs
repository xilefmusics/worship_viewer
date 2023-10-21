use super::database_migration;
use super::error::AppError;
use super::types::{
    Blob, BlobOcrUpdate, Collection, CollectionFetchedSongs, Group, Song, SongTitleUpdate, User,
    UserGroupsFetched, UserGroupsId,
};
use base64::{engine::general_purpose, Engine as _};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    url: String,
    auth: String,
    namespace: String,
    database: String,
}

impl Database {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
    ) -> Result<Self, AppError> {
        let url = format!("http://{}:{}/sql", host, port);
        let auth = format!(
            "Basic {}",
            general_purpose::STANDARD.encode(format!("{}:{}", username, password))
        );
        let namespace = namespace.into();
        let database = database.into();

        let database = Self {
            url,
            auth,
            namespace,
            database,
        };

        database_migration::migrate(&database).await?;

        Ok(database)
    }

    pub async fn query_string(&self, query: String) -> Result<String, AppError> {
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
        let response_string = self.query_string(query.clone()).await?;

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
                serde_json::to_string(&users)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?;

        let _: Vec<RelateResponse> = self.query_vec(relate_query).await?;

        Ok(created_users)
    }

    pub async fn get_blobs(&self, username: &str) -> Result<Vec<Blob>, AppError> {
        self.query_vec(
            format!(
                "SELECT * FROM blob WHERE (group<-member_of<-user).name contains \"{}\";",
                username
            )
            .into(),
        )
        .await
    }

    pub async fn get_blob(&self, username: &str, id: &str) -> Result<Blob, AppError> {
        self.query_one(format!("SELECT * FROM blob WHERE id = {} AND (group<-member_of<-user).name contains \"{}\";", id, username).into()).await
    }

    pub async fn add_blobs(&self, blobs: &Vec<Blob>) -> Result<Vec<Blob>, AppError> {
        Ok(self
            .query_vec(format!(
                "INSERT INTO blob {};",
                serde_json::to_string(blobs)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?)
    }

    pub async fn update_blobs_ocr(
        &self,
        ocr_update: &Vec<BlobOcrUpdate>,
    ) -> Result<Vec<Blob>, AppError> {
        let sql = ocr_update
            .into_iter()
            .map(|ocr_update| format!("UPDATE {} SET ocr = \"{}\";", ocr_update.id, ocr_update.ocr))
            .collect::<String>();
        Ok(self.query_vec(sql).await?)
    }

    pub async fn get_songs(&self, username: &str) -> Result<Vec<Song>, AppError> {
        self.query_vec(
            format!(
                "SELECT *, (SELECT title FROM $parent.collection)[0].title as collection FROM song WHERE (group<-member_of<-user).name contains \"{}\" ORDER BY title;",
                username
            )
            .into(),
        )
        .await
    }

    pub async fn get_song(&self, username: &str, id: &str) -> Result<Song, AppError> {
        self.query_one(format!("SELECT * FROM song WHERE (group<-member_of<-user).name contains \"{}\" AND id = {};", username, id).into()).await
    }

    pub async fn add_songs(&self, songs: &Vec<Song>) -> Result<Vec<Song>, AppError> {
        Ok(self
            .query_vec(format!(
                "INSERT INTO song {};",
                serde_json::to_string(&songs)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?)
    }

    pub async fn update_songs_title(
        &self,
        title_update: &Vec<SongTitleUpdate>,
    ) -> Result<Vec<Song>, AppError> {
        let sql = title_update
            .into_iter()
            .map(|title_update| {
                format!(
                    "UPDATE {} SET title = \"{}\";",
                    title_update.id, title_update.title
                )
            })
            .collect::<String>();
        Ok(self.query_vec(sql).await?)
    }

    pub async fn get_collections(&self, username: &str) -> Result<Vec<Collection>, AppError> {
        self.query_vec(
            format!(
                "SELECT * FROM collection WHERE (group<-member_of<-user).name contains \"{}\";",
                username
            )
            .into(),
        )
        .await
    }

    pub async fn get_collection(&self, username: &str, id: &str) -> Result<Collection, AppError> {
        self.query_one(format!("SELECT * FROM collection WHERE (group<-member_of<-user).name contains \"{}\" AND id = {};", username, id).into()).await
    }

    pub async fn get_collection_fetched_songs(
        &self,
        username: &str,
        id: &str,
    ) -> Result<CollectionFetchedSongs, AppError> {
        self.query_one(format!("SELECT * FROM collection WHERE (group<-member_of<-user).name contains \"{}\" AND id = {} FETCH songs;", username, id).into()).await
    }

    pub async fn add_collections(
        &self,
        collections: &Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(self
            .query_vec(format!(
                "INSERT INTO collection {};",
                serde_json::to_string(&collections)
                    .map_err(|err| AppError::Database(format!("{}", err)))?
            ))
            .await?)
    }

    pub async fn check_user_admin(&self, username: &str) -> Result<bool, AppError> {
        self.query_check::<SimpleValue<String>>(format!("SELECT name as value FROM user WHERE name = \"{}\" AND (->member_of->group).name CONTAINS \"admin\";",username)).await
    }
}
