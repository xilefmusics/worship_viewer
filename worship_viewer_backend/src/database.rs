use crate::error::AppError;
use crate::settings::Settings;
use crate::types::IdGetter;

use futures::future::join_all;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn new(settings: Settings) -> Self {
        let client = Surreal::new::<Ws>(format!("{}:{}", settings.db_host, settings.db_port))
            .await
            .unwrap();
        client
            .signin(Root {
                username: &settings.db_user,
                password: &settings.db_password,
            })
            .await
            .unwrap();
        client
            .use_ns(settings.db_namespace)
            .use_db(settings.db_database)
            .await
            .unwrap();
        Self { client }
    }

    pub async fn create<T: Serialize + DeserializeOwned + Clone + std::fmt::Debug + IdGetter>(
        &self,
        table: &str,
        data: T,
    ) -> Result<T, AppError> {
        self.client
            .create((table, data.get_id_second()))
            .content(data)
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .ok_or(AppError::Database("record is none".into()))
    }

    pub async fn create_vec<
        T: Serialize + DeserializeOwned + Clone + std::fmt::Debug + IdGetter,
    >(
        &self,
        table: &str,
        data: Vec<T>,
    ) -> Result<Vec<T>, AppError> {
        join_all(data.into_iter().map(|data| self.create(table, data)))
            .await
            .into_iter()
            .collect()
    }

    pub async fn select_all_with_pagination<
        T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
    >(
        &self,
        table: &str,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<T>, AppError> {
        self.client
            .query(
                "SELECT * FROM type::table($table) LIMIT type::int($limit) START type::int($start)",
            )
            .bind(("table", table))
            .bind(("limit", page_size))
            .bind(("start", page_size * page))
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .take(0)
            .map_err(|err| AppError::Database(format!("{}", err)))
    }
}
