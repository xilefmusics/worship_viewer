use crate::error::AppError;

use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub struct Select<'a> {
    client: &'a Surreal<Client>,
    table: Option<&'a str>,
    limit: Option<usize>,
    start: Option<usize>,
    fetch: Option<&'a str>,
    where_conditions: Vec<String>,
}

impl<'a> Select<'a> {
    pub fn new(client: &'a Surreal<Client>) -> Self {
        Self {
            client,
            table: None,
            limit: None,
            start: None,
            where_conditions: vec![],
            fetch: None,
        }
    }

    // TODO: tablecheck at compiletime
    pub fn table(mut self, table: &'a str) -> Self {
        self.table = Some(table);
        self
    }

    // TODO: also possible to set page and page_size
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = Some(start);
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

        if let Some(limit) = self.limit {
            query = format!("{} LIMIT {}", query, limit);
        }

        if let Some(start) = self.start {
            query = format!("{} START {}", query, start);
        }

        if self.where_conditions.len() > 0 {
            query = format!("{} WHERE {}", query, self.where_conditions.join(" AND "));
        }

        if let Some(fetch) = self.fetch.clone() {
            query = format!("{} FETCH {}", query, fetch);
        }

        Ok(query)
    }

    pub async fn query<T: Serialize + DeserializeOwned + Clone + std::fmt::Debug>(
        &self,
    ) -> Result<Vec<T>, AppError> {
        self.client
            .query(dbg!(self.serialize()?))
            .await
            .map_err(|err| AppError::Database(format!("{}", err)))?
            .take(0)
            .map_err(|err| AppError::Database(format!("{}", err)))
    }
}
