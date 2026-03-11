use crate::auth::otp::{OtpRequest, OtpVerify};
use crate::blob::{Blob, CreateBlob};
use crate::collection::{Collection, CreateCollection};
use crate::error::NetworkClientError;
use crate::like::LikeStatus;
#[cfg(any(
    all(feature = "cli", not(target_arch = "wasm32")),
    all(feature = "frontend", target_arch = "wasm32")
))]
use crate::net::{DefaultHttpClient, HttpClientConfig};
use crate::net::HttpClient;
use crate::player::Player;
use crate::setlist::{CreateSetlist, Setlist};
use crate::song::{CreateSong, Song};
use crate::user::{CreateUserRequest, Session, User};
use std::vec::Vec;
pub struct ApiClient<C: HttpClient> {
    client: C,
}

impl<C: HttpClient> ApiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

#[cfg(any(
    all(feature = "cli", not(target_arch = "wasm32")),
    all(feature = "frontend", target_arch = "wasm32")
))]
impl ApiClient<DefaultHttpClient> {
    pub fn with_default(config: HttpClientConfig) -> Self {
        Self {
            client: DefaultHttpClient::new(config),
        }
    }
}

impl<C: HttpClient> ApiClient<C> {
    pub async fn request_otp(&self, payload: OtpRequest) -> Result<(), NetworkClientError> {
        self.client
            .post_no_response("auth/otp/request", &payload)
            .await
    }

    pub async fn verify_otp(&self, payload: OtpVerify) -> Result<Session, NetworkClientError> {
        self.client.post("auth/otp/verify", &payload).await
    }

    pub async fn get_openapi_docs(
        &self,
    ) -> Result<serde_json::Value, NetworkClientError> {
        self.client.get("api/docs/openapi.json").await
    }

    pub async fn logout(&self) -> Result<(), NetworkClientError> {
        self.client
            .post_no_response("auth/logout", &serde_json::json!({}))
            .await
    }

    pub async fn get_current_user(&self) -> Result<User, NetworkClientError> {
        self.client.get("api/v1/users/me").await
    }

    pub async fn get_user(&self, id: &str) -> Result<User, NetworkClientError> {
        self.client.get(&format!("api/v1/users/{id}")).await
    }

    pub async fn list_users(&self) -> Result<Vec<User>, NetworkClientError> {
        self.client.get("api/v1/users").await
    }

    pub async fn create_user(
        &self,
        payload: CreateUserRequest,
    ) -> Result<User, NetworkClientError> {
        self.client.post("api/v1/users", &payload).await
    }

    pub async fn delete_user(&self, id: &str) -> Result<User, NetworkClientError> {
        self.client.delete(&format!("api/v1/users/{id}")).await
    }

    pub async fn list_my_sessions(&self) -> Result<Vec<Session>, NetworkClientError> {
        self.client.get("api/v1/users/me/sessions").await
    }

    pub async fn get_my_session(&self, id: &str) -> Result<Session, NetworkClientError> {
        self.client
            .get(&format!("api/v1/users/me/sessions/{id}"))
            .await
    }

    pub async fn delete_my_session(&self, id: &str) -> Result<Session, NetworkClientError> {
        self.client
            .delete(&format!("api/v1/users/me/sessions/{id}"))
            .await
    }

    pub async fn create_session_for_user(
        &self,
        user_id: &str,
    ) -> Result<Session, NetworkClientError> {
        self.client
            .post(
                &format!("api/v1/users/{user_id}/sessions"),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn list_sessions_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Session>, NetworkClientError> {
        self.client
            .get(&format!("api/v1/users/{user_id}/sessions"))
            .await
    }

    pub async fn get_session_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Session, NetworkClientError> {
        self.client
            .get(&format!("api/v1/users/{user_id}/sessions/{id}"))
            .await
    }

    pub async fn delete_session_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Session, NetworkClientError> {
        self.client
            .delete(&format!("api/v1/users/{user_id}/sessions/{id}"))
            .await
    }

    pub async fn get_songs(&self) -> Result<Vec<Song>, NetworkClientError> {
        self.client.get("api/v1/songs").await
    }

    pub async fn get_song(&self, id: &str) -> Result<Song, NetworkClientError> {
        self.client.get(&format!("api/v1/songs/{id}")).await
    }

    pub async fn get_song_player(&self, id: &str) -> Result<Player, NetworkClientError> {
        self.client.get(&format!("api/v1/songs/{id}/player")).await
    }

    pub async fn get_song_export_url(&self, id: &str, format: &str) -> String {
        format!("api/v1/songs/{id}/export?format={format}")
    }

    pub async fn create_song(&self, payload: CreateSong) -> Result<Song, NetworkClientError> {
        self.client.post("api/v1/songs", &payload).await
    }

    pub async fn update_song(
        &self,
        id: &str,
        payload: CreateSong,
    ) -> Result<Song, NetworkClientError> {
        self.client
            .put(&format!("api/v1/songs/{id}"), &payload)
            .await
    }

    pub async fn delete_song(&self, id: &str) -> Result<Song, NetworkClientError> {
        self.client.delete(&format!("api/v1/songs/{id}")).await
    }

    pub async fn import_song(&self, identifier: &str) -> Result<Song, NetworkClientError> {
        self.client
            .get(&format!("api/v1/songs/import/{identifier}"))
            .await
    }

    pub async fn get_song_like_status(&self, id: &str) -> Result<bool, NetworkClientError> {
        self.client
            .get(&format!("api/v1/songs/{id}/likes"))
            .await
            .map(|like: LikeStatus| like.liked)
    }

    pub async fn update_song_like_status(
        &self,
        id: &str,
        liked: bool,
    ) -> Result<bool, NetworkClientError> {
        self.client
            .put(&format!("api/v1/songs/{id}/likes"), &LikeStatus { liked })
            .await
            .map(|like: LikeStatus| like.liked)
    }

    pub async fn list_collections(&self) -> Result<Vec<Collection>, NetworkClientError> {
        self.client.get("api/v1/collections").await
    }

    pub async fn get_collection(&self, id: &str) -> Result<Collection, NetworkClientError> {
        self.client.get(&format!("api/v1/collections/{id}")).await
    }

    pub async fn get_collection_songs(&self, id: &str) -> Result<Vec<Song>, NetworkClientError> {
        self.client
            .get(&format!("api/v1/collections/{id}/songs"))
            .await
    }

    pub async fn get_collection_player(&self, id: &str) -> Result<Player, NetworkClientError> {
        self.client
            .get(&format!("api/v1/collections/{id}/player"))
            .await
    }

    pub async fn get_collection_export_url(&self, id: &str, format: &str) -> String {
        format!("api/v1/collections/{id}/export?format={format}")
    }

    pub async fn create_collection(
        &self,
        payload: CreateCollection,
    ) -> Result<Collection, NetworkClientError> {
        self.client.post("api/v1/collections", &payload).await
    }

    pub async fn update_collection(
        &self,
        id: &str,
        payload: CreateCollection,
    ) -> Result<Collection, NetworkClientError> {
        self.client
            .put(&format!("api/v1/collections/{id}"), &payload)
            .await
    }

    pub async fn delete_collection(&self, id: &str) -> Result<Collection, NetworkClientError> {
        self.client
            .delete(&format!("api/v1/collections/{id}"))
            .await
    }

    pub async fn list_setlists(&self) -> Result<Vec<Setlist>, NetworkClientError> {
        self.client.get("api/v1/setlists").await
    }

    pub async fn get_setlist(&self, id: &str) -> Result<Setlist, NetworkClientError> {
        self.client.get(&format!("api/v1/setlists/{id}")).await
    }

    pub async fn get_setlist_songs(&self, id: &str) -> Result<Vec<Song>, NetworkClientError> {
        self.client
            .get(&format!("api/v1/setlists/{id}/songs"))
            .await
    }

    pub async fn get_setlist_player(&self, id: &str) -> Result<Player, NetworkClientError> {
        self.client
            .get(&format!("api/v1/setlists/{id}/player"))
            .await
    }

    pub async fn get_setlist_export_url(&self, id: &str, format: &str) -> String {
        format!("api/v1/setlists/{id}/export?format={format}")
    }

    pub async fn create_setlist(
        &self,
        payload: CreateSetlist,
    ) -> Result<Setlist, NetworkClientError> {
        self.client.post("api/v1/setlists", &payload).await
    }

    pub async fn update_setlist(
        &self,
        id: &str,
        payload: CreateSetlist,
    ) -> Result<Setlist, NetworkClientError> {
        self.client
            .put(&format!("api/v1/setlists/{id}"), &payload)
            .await
    }

    pub async fn delete_setlist(&self, id: &str) -> Result<Setlist, NetworkClientError> {
        self.client.delete(&format!("api/v1/setlists/{id}")).await
    }

    pub async fn list_blobs(&self) -> Result<Vec<Blob>, NetworkClientError> {
        self.client.get("api/v1/blobs").await
    }

    pub async fn get_blob(&self, id: &str) -> Result<Blob, NetworkClientError> {
        self.client.get(&format!("api/v1/blobs/{id}")).await
    }

    pub async fn create_blob(&self, payload: CreateBlob) -> Result<Blob, NetworkClientError> {
        self.client.post("api/v1/blobs", &payload).await
    }

    pub async fn update_blob(
        &self,
        id: &str,
        payload: CreateBlob,
    ) -> Result<Blob, NetworkClientError> {
        self.client
            .put(&format!("api/v1/blobs/{id}"), &payload)
            .await
    }

    pub async fn delete_blob(&self, id: &str) -> Result<Blob, NetworkClientError> {
        self.client.delete(&format!("api/v1/blobs/{id}")).await
    }

    pub async fn download_blob_image_url(&self, id: &str) -> String {
        format!("api/v1/blobs/{id}/data")
    }
}
