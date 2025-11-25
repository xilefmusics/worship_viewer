use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use yew_router::prelude::Navigator;

use shared::auth::otp::{OtpRequest, OtpVerify};
use shared::blob::Blob;
use shared::blob::CreateBlob;
use shared::collection::Collection;
use shared::collection::CreateCollection;
use shared::error::ErrorResponse;
use shared::like::LikeStatus;
use shared::player::Player;
use shared::setlist::CreateSetlist;
use shared::setlist::Setlist;
use shared::song::CreateSong;
use shared::song::Song;
use shared::user::{CreateUserRequest, Session, User};

use super::ApiError;
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

    async fn map_error(&self, response: Response) -> ApiError {
        let status = response.status();
        let error = response
            .json::<ErrorResponse>()
            .await
            .map(|resp| ApiError::from_error_response(status, resp))
            .unwrap_or_else(|err| {
                ApiError::InternalServerError(format!("Failed to parse error response: {err}"))
            });

        match error {
            ApiError::Unauthorized(msg) => {
                self.route_logout();
                ApiError::Unauthorized(msg)
            }
            err => err,
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
            _ => Err(self.map_error(response).await),
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
            _ => Err(self.map_error(response).await),
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
            _ => Err(self.map_error(response).await),
        }
    }
    async fn put<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Default,
        B: ?Sized + Serialize,
    {
        let response = Request::put(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .json(body)?
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Ok(T::default()),
            _ => Err(self.map_error(response).await),
        }
    }

    async fn delete<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned + Default,
    {
        let response = Request::delete(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Ok(T::default()),
            _ => Err(self.map_error(response).await),
        }
    }

    async fn get_entity<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = Request::get(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Err(ApiError::InternalServerError(
                "Unexpected 204 response for entity retrieval".into(),
            )),
            _ => Err(self.map_error(response).await),
        }
    }

    async fn put_entity<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        B: ?Sized + Serialize,
    {
        let response = Request::put(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .json(body)?
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Err(ApiError::InternalServerError(
                "Unexpected 204 response for entity update".into(),
            )),
            _ => Err(self.map_error(response).await),
        }
    }

    async fn post_entity<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        B: ?Sized + Serialize,
    {
        let response = Request::post(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .json(body)?
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Err(ApiError::InternalServerError(
                "Unexpected 204 response for entity creation".into(),
            )),
            _ => Err(self.map_error(response).await),
        }
    }

    async fn delete_entity<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = Request::delete(&Self::build_path(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await?;

        match response.status() {
            200 | 201 => Ok(response.json::<T>().await?),
            204 => Err(ApiError::InternalServerError(
                "Unexpected 204 response for entity deletion".into(),
            )),
            _ => Err(self.map_error(response).await),
        }
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

    #[allow(dead_code)]
    pub async fn request_otp(&self, email: String) -> Result<(), ApiError> {
        self.post("/auth/otp/request", &OtpRequest { email }).await
    }

    #[allow(dead_code)]
    pub async fn verify_otp(&self, email: String, code: String) -> Result<Session, ApiError> {
        self.post("/auth/otp/verify", &OtpVerify { email, code })
            .await
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub async fn get_users(&self) -> Result<Vec<User>, ApiError> {
        self.get("/api/v1/users").await
    }

    #[allow(dead_code)]
    pub async fn get_user(&self, id: &str) -> Result<User, ApiError> {
        self.get(&format!("/api/v1/users/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn create_user(&self, payload: &CreateUserRequest) -> Result<User, ApiError> {
        self.post("/api/v1/users", payload).await
    }

    #[allow(dead_code)]
    pub async fn delete_user(&self, id: &str) -> Result<User, ApiError> {
        self.delete(&format!("/api/v1/users/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_users_me(&self) -> Result<User, ApiError> {
        self.get("/api/v1/users/me").await
    }

    #[allow(dead_code)]
    pub async fn get_sessions_for_current_user(&self) -> Result<Vec<Session>, ApiError> {
        self.get("/api/v1/users/me/sessions").await
    }

    #[allow(dead_code)]
    pub async fn get_session_for_current_user(&self, id: &str) -> Result<Session, ApiError> {
        self.get(&format!("/api/v1/users/me/sessions/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn delete_session_for_current_user(&self, id: &str) -> Result<Session, ApiError> {
        self.delete(&format!("/api/v1/users/me/sessions/{}", id))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_sessions_for_user(&self, user_id: &str) -> Result<Vec<Session>, ApiError> {
        self.get(&format!("/api/v1/users/{}/sessions", user_id))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_session_for_user(&self, user_id: &str, id: &str) -> Result<Session, ApiError> {
        self.get(&format!("/api/v1/users/{}/sessions/{}", user_id, id))
            .await
    }

    #[allow(dead_code)]
    pub async fn create_session_for_user(&self, user_id: &str) -> Result<Session, ApiError> {
        self.post_empty(&format!("/api/v1/users/{}/sessions", user_id))
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_session_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Session, ApiError> {
        self.delete(&format!("/api/v1/users/{}/sessions/{}", user_id, id))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_songs(&self) -> Result<Vec<Song>, ApiError> {
        self.get("/api/v1/songs").await
    }

    #[allow(dead_code)]
    pub async fn get_song(&self, id: &str) -> Result<Song, ApiError> {
        self.get(&format!("/api/v1/songs/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_song_player(&self, id: &str) -> Result<Player, ApiError> {
        self.get(&format!("/api/v1/songs/{}/player", id)).await
    }

    #[allow(dead_code)]
    pub fn get_song_export_url(&self, id: &str, format: &str) -> String {
        Self::build_path(&format!("/api/v1/songs/{}/export?format={}", id, format))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_song(&self, payload: &CreateSong) -> Result<Song, ApiError> {
        self.post_entity("/api/v1/songs", payload).await
    }

    #[allow(dead_code)]
    pub async fn update_song(&self, id: &str, payload: &CreateSong) -> Result<Song, ApiError> {
        self.put_entity(&format!("/api/v1/songs/{}", id), payload)
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_song(&self, id: &str) -> Result<Song, ApiError> {
        self.delete_entity(&format!("/api/v1/songs/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_song_like_status(&self, id: &str) -> Result<bool, ApiError> {
        self.get(&format!("/api/v1/songs/{}/likes", id))
            .await
            .map(|like: LikeStatus| like.liked)
    }

    #[allow(dead_code)]
    pub async fn update_song_like_status(&self, id: &str, liked: bool) -> Result<bool, ApiError> {
        self.put(
            &format!("/api/v1/songs/{}/likes", id),
            &LikeStatus { liked },
        )
        .await
        .map(|like: LikeStatus| like.liked)
    }

    #[allow(dead_code)]
    pub async fn get_collections(&self) -> Result<Vec<Collection>, ApiError> {
        self.get("/api/v1/collections").await
    }

    #[allow(dead_code)]
    pub async fn get_collection(&self, id: &str) -> Result<Collection, ApiError> {
        self.get(&format!("/api/v1/collections/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_collection_songs(&self, id: &str) -> Result<Vec<Song>, ApiError> {
        self.get(&format!("/api/v1/collections/{}/songs", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_collection_player(&self, id: &str) -> Result<Player, ApiError> {
        self.get(&format!("/api/v1/collections/{}/player", id))
            .await
    }

    #[allow(dead_code)]
    pub fn get_collection_export_url(&self, id: &str, format: &str) -> String {
        Self::build_path(&format!(
            "/api/v1/collections/{}/export?format={}",
            id, format
        ))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_collection(
        &self,
        payload: &CreateCollection,
    ) -> Result<Collection, ApiError> {
        self.post_entity("/api/v1/collections", payload).await
    }

    #[allow(dead_code)]
    pub async fn update_collection(
        &self,
        id: &str,
        payload: &CreateCollection,
    ) -> Result<Collection, ApiError> {
        self.put_entity(&format!("/api/v1/collections/{}", id), payload)
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_collection(&self, id: &str) -> Result<Collection, ApiError> {
        self.delete_entity(&format!("/api/v1/collections/{}", id))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_setlists(&self) -> Result<Vec<Setlist>, ApiError> {
        self.get("/api/v1/setlists").await
    }

    #[allow(dead_code)]
    pub async fn get_setlist(&self, id: &str) -> Result<Setlist, ApiError> {
        self.get(&format!("/api/v1/setlists/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_setlist_songs(&self, id: &str) -> Result<Vec<Song>, ApiError> {
        self.get(&format!("/api/v1/setlists/{}/songs", id)).await
    }

    #[allow(dead_code)]
    pub async fn get_setlist_player(&self, id: &str) -> Result<Player, ApiError> {
        self.get(&format!("/api/v1/setlists/{}/player", id)).await
    }

    #[allow(dead_code)]
    pub fn get_setlist_export_url(&self, id: &str, format: &str) -> String {
        Self::build_path(&format!("/api/v1/setlists/{}/export?format={}", id, format))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_setlist(&self, payload: &CreateSetlist) -> Result<Setlist, ApiError> {
        self.post_entity("/api/v1/setlists", payload).await
    }

    #[allow(dead_code)]
    pub async fn update_setlist(
        &self,
        id: &str,
        payload: &CreateSetlist,
    ) -> Result<Setlist, ApiError> {
        self.put_entity(&format!("/api/v1/setlists/{}", id), payload)
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_setlist(&self, id: &str) -> Result<Setlist, ApiError> {
        self.delete_entity(&format!("/api/v1/setlists/{}", id))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_blobs(&self) -> Result<Vec<Blob>, ApiError> {
        self.get("/api/v1/blobs").await
    }

    #[allow(dead_code)]
    pub async fn get_blob(&self, id: &str) -> Result<Blob, ApiError> {
        self.get_entity(&format!("/api/v1/blobs/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn create_blob(&self, payload: &CreateBlob) -> Result<Blob, ApiError> {
        self.post_entity("/api/v1/blobs", payload).await
    }

    #[allow(dead_code)]
    pub async fn update_blob(&self, id: &str, payload: &CreateBlob) -> Result<Blob, ApiError> {
        self.put_entity(&format!("/api/v1/blobs/{}", id), payload)
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_blob(&self, id: &str) -> Result<Blob, ApiError> {
        self.delete_entity(&format!("/api/v1/blobs/{}", id)).await
    }

    #[allow(dead_code)]
    pub async fn import_song_ultimate_guitar(&self, url: &str) -> Result<Song, ApiError> {
        let url = Url::parse(&url).unwrap();
        self.get(&format!(
            "/api/v1/songs/import/{}{}",
            url.host_str().unwrap_or("unknown").replace(".", "/"),
            url.path()
        ))
        .await
    }
}
