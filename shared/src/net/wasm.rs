use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{HttpClient, HttpClientConfig};
use crate::error::NetworkClientError;
use web_sys::RequestCredentials;

#[derive(Clone)]
pub struct WasmHttpClient {
    config: HttpClientConfig,
}

impl WasmHttpClient {
    pub fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }

    fn make_url(&self, path: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

#[async_trait::async_trait(?Send)]
impl HttpClient for WasmHttpClient {
    async fn get<T>(&self, path: &str) -> Result<T, NetworkClientError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let response = gloo_net::http::Request::get(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !(200..300).contains(&status) {
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        let value = serde_json::from_str::<T>(&text)?;
        Ok(value)
    }

    async fn post<B, T>(&self, path: &str, body: &B) -> Result<T, NetworkClientError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let payload = serde_json::to_string(body)?;
        let response = gloo_net::http::Request::post(&url)
            .header("Content-Type", "application/json")
            .credentials(RequestCredentials::Include)
            .body(payload)?
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !(200..300).contains(&status) {
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        let value = serde_json::from_str::<T>(&text)?;
        Ok(value)
    }

    async fn put<B, T>(&self, path: &str, body: &B) -> Result<T, NetworkClientError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let payload = serde_json::to_string(body)?;
        let response = gloo_net::http::Request::put(&url)
            .header("Content-Type", "application/json")
            .credentials(RequestCredentials::Include)
            .body(payload)?
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !(200..300).contains(&status) {
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        let value = serde_json::from_str::<T>(&text)?;
        Ok(value)
    }

    async fn patch<B, T>(&self, path: &str, body: &B) -> Result<T, NetworkClientError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let payload = serde_json::to_string(body)?;
        let response = gloo_net::http::Request::patch(&url)
            .header("Content-Type", "application/json")
            .credentials(RequestCredentials::Include)
            .body(payload)?
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !(200..300).contains(&status) {
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        let value = serde_json::from_str::<T>(&text)?;
        Ok(value)
    }

    async fn post_no_response<B>(&self, path: &str, body: &B) -> Result<(), NetworkClientError>
    where
        B: Serialize + Send + Sync,
    {
        let url = self.make_url(path);

        let payload = serde_json::to_string(body)?;
        let response = gloo_net::http::Request::post(&url)
            .header("Content-Type", "application/json")
            .credentials(RequestCredentials::Include)
            .body(payload)?
            .send()
            .await?;

        let status = response.status();
        if !(200..300).contains(&status) {
            let text = response.text().await.unwrap_or_default();
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        Ok(())
    }

    async fn delete<T>(&self, path: &str) -> Result<T, NetworkClientError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.make_url(path);

        let response = gloo_net::http::Request::delete(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !(200..300).contains(&status) {
            return Err(NetworkClientError::RequestFailed {
                status: Some(status as u16),
                message: text,
            });
        }

        let value = serde_json::from_str::<T>(&text)?;
        Ok(value)
    }
}
