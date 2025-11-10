use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};
use yew_router::prelude::Navigator;

use super::{ApiError, ErrorResponse, OtpRequestPayload, OtpVerifyPayload, Session, User};
use crate::route::Route;

#[derive(Clone, PartialEq)]
pub struct Api {
    navigator: Navigator,
}

impl Api {
    pub fn new(navigator: Navigator) -> Self {
        Self { navigator }
    }

    fn build_path(path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        }
    }

    async fn get<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Default,
    {
        let response = Request::get(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Ok(T::default()),
            status => Err(
                match response.json::<ErrorResponse>().await?.to_api_error(status) {
                    ApiError::Unauthorized(msg) => {
                        self.route_logout();
                        ApiError::Unauthorized(msg)
                    }
                    err => err,
                },
            ),
        }
    }

    async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Default,
        B: ?Sized + Serialize,
    {
        let response = Request::post(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .json(body)?
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Ok(T::default()),
            status => Err(
                match response.json::<ErrorResponse>().await?.to_api_error(status) {
                    ApiError::Unauthorized(msg) => {
                        self.route_logout();
                        ApiError::Unauthorized(msg)
                    }
                    err => err,
                },
            ),
        }
    }

    async fn post_empty<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Default,
    {
        let response = Request::post(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Ok(T::default()),
            status => Err(
                match response.json::<ErrorResponse>().await?.to_api_error(status) {
                    ApiError::Unauthorized(msg) => {
                        self.route_logout();
                        ApiError::Unauthorized(msg)
                    }
                    err => err,
                },
            ),
        }
    }

    pub async fn get_users_me(&self) -> Result<User, ApiError> {
        self.get("/api/v1/users/me").await
    }

    pub async fn request_otp(&self, email: String) -> Result<(), ApiError> {
        self.post("/auth/otp/request", &OtpRequestPayload { email })
            .await
    }

    pub async fn verify_otp(&self, email: String, code: String) -> Result<Session, ApiError> {
        self.post("/auth/otp/verify", &OtpVerifyPayload { email, code })
            .await
    }

    pub fn auth_login_url(&self, provider: Option<&str>) -> String {
        match provider {
            Some(provider) if !provider.is_empty() => {
                format!("/auth/login?provider={}", provider)
            }
            _ => "/auth/login".into(),
        }
    }

    pub async fn logout(&self) -> Result<(), ApiError> {
        self.post_empty("/auth/logout").await
    }

    pub fn route_login(&self) {
        self.navigator.push(&Route::Login);
    }

    pub fn route_logout(&self) {
        self.navigator.push(&Route::Logout);
    }

    pub fn route_index(&self) {
        self.navigator.push(&Route::Index);
    }
}
