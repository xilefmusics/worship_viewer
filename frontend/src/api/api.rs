use yew_router::prelude::Navigator;

use shared::auth::otp::{OtpRequest, OtpVerify};
use shared::blob::Blob;
use shared::blob::CreateBlob;
use shared::collection::Collection;
use shared::collection::CreateCollection;
use shared::error::NetworkClientError;
use shared::net::{DefaultHttpClient, HttpClientConfig};
use shared::player::Player;
use shared::setlist::CreateSetlist;
use shared::setlist::Setlist;
use shared::song::CreateSong;
use shared::song::Song;
use shared::user::{CreateUserRequest, Session, User};

use super::error::{ApiError, OperationType};
use crate::route::Route;
use shared::api::{ApiClient, ListQuery};

use std::rc::Rc;

#[derive(Clone)]
pub struct Api {
    client: Rc<ApiClient<DefaultHttpClient>>,
    navigator: Navigator,
}

impl PartialEq for Api {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.client, &other.client)
    }
}

impl Api {
    pub fn new(navigator: Navigator, base_url: String) -> Self {
        let config = HttpClientConfig {
            base_url,
            timeout: None,
            session_cookie: None,
            bearer_token: None,
        };
        let client = Rc::new(ApiClient::with_default(config));

        Self { client, navigator }
    }

    fn build_path(path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        }
    }

    fn handle_error(&self, err: NetworkClientError) -> ApiError {
        let api_error: ApiError = err.into();
        match api_error {
            ApiError::Unauthorized(msg) => {
                self.route_logout();
                ApiError::Unauthorized(msg)
            }
            other => other,
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
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .request_otp(OtpRequest { email })
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn verify_otp(&self, email: String, code: String) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .verify_otp(OtpVerify { email, code })
            .await
            .map_err(|e| self.handle_error(e))
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
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client.logout().await.map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_users(&self) -> Result<Vec<User>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_users(ListQuery::default())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_user(&self, id: &str) -> Result<User, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_user(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn create_user(&self, payload: &CreateUserRequest) -> Result<User, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_user(payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_user(&self, id: &str) -> Result<User, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_user(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_users_me(&self) -> Result<User, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_current_user()
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_sessions_for_current_user(&self) -> Result<Vec<Session>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_my_sessions()
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_session_for_current_user(&self, id: &str) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_my_session(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_session_for_current_user(&self, id: &str) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_my_session(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_sessions_for_user(&self, user_id: &str) -> Result<Vec<Session>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_sessions_for_user(user_id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_session_for_user(&self, user_id: &str, id: &str) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_session_for_user(user_id, id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn create_session_for_user(&self, user_id: &str) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_session_for_user(user_id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_session_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Session, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_session_for_user(user_id, id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_songs(&self) -> Result<Vec<Song>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_songs(ListQuery::default())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_song(&self, id: &str) -> Result<Song, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_song(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_song_player(&self, id: &str) -> Result<Player, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_song_player(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_song(&self, payload: &CreateSong) -> Result<Song, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_song(payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn update_song(&self, id: &str, payload: &CreateSong) -> Result<Song, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .update_song(id, payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_song(&self, id: &str) -> Result<Song, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_song(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_song_like_status(&self, id: &str) -> Result<bool, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_song_like_status(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn update_song_like_status(&self, id: &str, liked: bool) -> Result<bool, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .update_song_like_status(id, liked)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_collections(&self) -> Result<Vec<Collection>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_collections(ListQuery::default())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_collection(&self, id: &str) -> Result<Collection, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_collection(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_collection_songs(&self, id: &str) -> Result<Vec<Song>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_collection_songs(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_collection_player(&self, id: &str) -> Result<Player, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_collection_player(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_collection(
        &self,
        payload: &CreateCollection,
    ) -> Result<Collection, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_collection(payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn update_collection(
        &self,
        id: &str,
        payload: &CreateCollection,
    ) -> Result<Collection, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .update_collection(id, payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_collection(&self, id: &str) -> Result<Collection, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_collection(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_setlists(&self) -> Result<Vec<Setlist>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_setlists(ListQuery::default())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_setlist(&self, id: &str) -> Result<Setlist, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_setlist(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_setlist_songs(&self, id: &str) -> Result<Vec<Song>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_setlist_songs(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_setlist_player(&self, id: &str) -> Result<Player, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_setlist_player(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub async fn create_setlist(&self, payload: &CreateSetlist) -> Result<Setlist, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_setlist(payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn update_setlist(
        &self,
        id: &str,
        payload: &CreateSetlist,
    ) -> Result<Setlist, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .update_setlist(id, payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_setlist(&self, id: &str) -> Result<Setlist, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_setlist(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_blobs(&self) -> Result<Vec<Blob>, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .list_blobs(ListQuery::default())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn get_blob(&self, id: &str) -> Result<Blob, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Read);
        self.client
            .get_blob(id)
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn create_blob(&self, payload: &CreateBlob) -> Result<Blob, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .create_blob(payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn update_blob(&self, id: &str, payload: &CreateBlob) -> Result<Blob, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .update_blob(id, payload.clone())
            .await
            .map_err(|e| self.handle_error(e))
    }

    #[allow(dead_code)]
    pub async fn delete_blob(&self, id: &str) -> Result<Blob, ApiError> {
        ApiError::check_and_notify_offline(OperationType::Write);
        self.client
            .delete_blob(id)
            .await
            .map_err(|e| self.handle_error(e))
    }
}
