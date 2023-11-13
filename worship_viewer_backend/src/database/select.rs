use crate::error::AppError;

use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub struct Select<'a> {
    client: &'a Surreal<Client>,
    table: Option<&'a str>,
    page: Option<usize>,
    page_size: Option<usize>,
    fetch: Option<&'a str>,
    where_conditions: Vec<String>,
}

impl<'a> Select<'a> {
    pub fn new(client: &'a Surreal<Client>) -> Self {
        Self {
            client,
            table: None,
            page: None,
            page_size: None,
            where_conditions: vec![],
            fetch: None,
        }
    }

    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    pub fn page_opt(mut self, page: Option<usize>) -> Self {
        self.page = page;
        self
    }

    pub fn page_size_opt(mut self, page_size: Option<usize>) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn fetch(mut self, fetch: &'a str) -> Self {
        self.fetch = Some(fetch);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.where_conditions.push(format!("id == {}", id));
        self
    }

    pub fn user(mut self, user: &str) -> Self {
        self.where_conditions
            .push(format!("group in user:{}.groups", user));
        self
    }

    pub fn serialize(&self) -> Result<String, AppError> {
        let mut query = format!(
            "SELECT * FROM {}",
            self.table
                .clone()
                .ok_or(AppError::Other("no table given".into()))?
        )
        .into();

        if self.where_conditions.len() > 0 {
            query = format!("{} WHERE {}", query, self.where_conditions.join(" AND "));
        }

        if let Some(fetch) = self.fetch.clone() {
            query = format!("{} FETCH {}", query, fetch);
        }

        let (limit, start) = if let Some(page) = self.page {
            if let Some(page_size) = self.page_size {
                (Some(page_size), Some(page * page_size))
            } else {
                (Some(100), Some(page * 100))
            }
        } else {
            if let Some(page_size) = self.page_size {
                (Some(page_size), Some(0))
            } else {
                (None, None)
            }
        };

        if let Some(limit) = limit {
            query = format!("{} LIMIT {}", query, limit);
        }

        if let Some(start) = start {
            query = format!("{} START {}", query, start);
        }

        Ok(query)
    }

    pub async fn query<T: Serialize + DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
    ) -> Result<Vec<T>, AppError> {
        self.client
            .query(self.serialize()?)
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .take(0)
            .map_err(|err| AppError::Database(format!("{}", err)))
    }

    pub async fn query_one<T: Serialize + DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
    ) -> Result<T, AppError> {
        Ok(self.query::<T>().await?.remove(0))
    }
}
