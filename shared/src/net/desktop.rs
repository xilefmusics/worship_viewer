use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{HttpClient, HttpClientConfig};
use crate::error::NetworkClientError;

#[derive(Clone)]
pub struct DesktopHttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl DesktopHttpClient {
    pub fn new(config: HttpClientConfig) -> Self {
        let mut builder = Client::builder();

        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        let client = builder.build().expect("failed to build reqwest client");

        Self { client, config }
    }

    fn make_url(&self, path: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

#[async_trait::async_trait]
impl HttpClient for DesktopHttpClient {
    async fn get<T>(&self, path: &str) -> Result<T, NetworkClientError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let mut request = self.client.get(url);
        if let Some(cookie) = &self.config.session_cookie {
            request = request.header(reqwest::header::COOKIE, format!("sso_session={cookie}"));
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let response = response.error_for_status()?;
        let value = response.json::<T>().await?;

        Ok(value)
    }

    async fn post<B, T>(&self, path: &str, body: &B) -> Result<T, NetworkClientError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let mut request = self.client.post(url).json(body);
        if let Some(cookie) = &self.config.session_cookie {
            request = request.header(reqwest::header::COOKIE, format!("sso_session={cookie}"));
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let response = response.error_for_status()?;
        let value = response.json::<T>().await?;

        Ok(value)
    }

    async fn put<B, T>(&self, path: &str, body: &B) -> Result<T, NetworkClientError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let mut request = self.client.put(url).json(body);
        if let Some(cookie) = &self.config.session_cookie {
            request = request.header(reqwest::header::COOKIE, format!("sso_session={cookie}"));
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let response = response.error_for_status()?;
        let value = response.json::<T>().await?;

        Ok(value)
    }

    async fn post_no_response<B>(&self, path: &str, body: &B) -> Result<(), NetworkClientError>
    where
        B: Serialize + Send + Sync,
    {
        let url = self.make_url(path);

        let mut request = self.client.post(url).json(body);
        if let Some(cookie) = &self.config.session_cookie {
            request = request.header(reqwest::header::COOKIE, format!("sso_session={cookie}"));
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        response.error_for_status()?;

        Ok(())
    }

    async fn delete<T>(&self, path: &str) -> Result<T, NetworkClientError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let mut request = self.client.delete(url);
        if let Some(cookie) = &self.config.session_cookie {
            request = request.header(reqwest::header::COOKIE, format!("sso_session={cookie}"));
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let response = response.error_for_status()?;
        let value = response.json::<T>().await?;

        Ok(value)
    }
}
