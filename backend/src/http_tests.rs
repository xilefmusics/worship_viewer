//! Phase 4: HTTP-layer tests using `actix_web::test`.
//!
//! Tests are grouped into slices matching the implementation roadmap:
//! - Slice 4A: test harness helpers (this module's top-level helpers)
//! - Slice 4B: auth middleware (BLC-AUTH-001, BLC-AUTH-002)
//! - Slice 4C: OpenAPI endpoint (BLC-DOCS-001)
//! - Slice 4D: HTTP contract — invalid path IDs + idempotent DELETE (BLC-HTTP-001, BLC-HTTP-002)
//! - Slice 4E: user admin gates (BLC-USER-005, BLC-USER-006, BLC-USER-007, BLC-USER-009)
//! - Slice 4F: session admin gates (BLC-SESS-003, BLC-SESS-004, BLC-SESS-005, BLC-SESS-006, BLC-SESS-009)
//! - Slice 4G: list pagination HTTP validation (BLC-LP-004 through BLC-LP-009)
//!
//! # Middleware error note
//!
//! `actix_web::test::call_service` panics when the service returns `Err` instead of
//! `Ok(ServiceResponse)`. In production actix-web converts middleware errors via
//! `ResponseError::error_response()` at the server boundary, but the test harness
//! does not do this automatically. The `call_status!` macro handles both `Ok` and `Err`
//! cases so tests that exercise `RequireUser` / `RequireAdmin` do not panic.

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::{App, test};
use anyhow::Result as AnyResult;
use shared::user::Session;

use crate::database::Database;
use crate::docs;
use crate::resources;
use crate::resources::User;
use crate::settings::{CookieConfig, PrinterConfig};
use crate::test_helpers::{create_song_with_title, create_user, session_service, test_db};

// ─── Slice 4A: test harness helpers ──────────────────────────────────────────

/// Build an `actix_web::App` wired with all resource services and the docs scope.
///
/// Takes `Arc<Database>` by value so no lifetime is captured, allowing
/// `actix_web::test::init_service` (which requires `'static`) to work.
///
/// The app does **not** include `auth::rest::scope()` (needs OIDC clients) or
/// `frontend::rest::scope()` (needs a static file directory on disk).
fn build_app(
    db: Arc<Database>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    use crate::test_helpers::{
        blob_service, collection_service, invitation_service, session_service, setlist_service,
        song_service, team_service, user_service,
    };

    // Use a throwaway temp path for blob storage; blobs are not written in these tests.
    let blob_dir = std::env::temp_dir()
        .join("worship_viewer_http_tests_blobs")
        .to_string_lossy()
        .into_owned();

    let cookie_cfg = Data::new(CookieConfig {
        name: "sso_session".into(),
        secure: false,
        session_ttl_seconds: 3600,
        post_login_path: "/".into(),
    });

    let printer_cfg = Data::new(PrinterConfig {
        address: "http://localhost:3000".into(),
        api_key: "test".into(),
    });

    App::new()
        .app_data(Data::from(db.clone()))
        .app_data(Data::new(blob_service(&db, blob_dir)))
        .app_data(Data::new(collection_service(&db)))
        .app_data(Data::new(song_service(&db)))
        .app_data(Data::new(setlist_service(&db)))
        .app_data(Data::new(team_service(&db)))
        .app_data(Data::new(invitation_service(&db)))
        .app_data(Data::new(user_service(&db)))
        .app_data(Data::new(session_service(&db)))
        .app_data(cookie_cfg)
        .app_data(printer_cfg)
        .service(docs::rest::scope())
        .service(resources::rest::scope(20 * 1024 * 1024))
}

/// Create a session for `user` and return its raw ID (used as Bearer token).
async fn create_session_token(db: &Arc<Database>, user: User) -> AnyResult<String> {
    let session = session_service(db)
        .create_session(Session::new(user, 3600))
        .await?;
    Ok(session.id)
}

/// Call the service with the given request and return the HTTP status code.
///
/// Unlike `actix_web::test::call_service`, this macro handles the case where the
/// service returns `Err(actix_web::Error)` (e.g. from `RequireUser` or `RequireAdmin`
/// middleware) by converting the error to its response status code via `ResponseError`.
macro_rules! call_status {
    ($app:expr, $req:expr) => {{
        use actix_web::dev::Service as _;
        match $app.call($req.to_request()).await {
            Ok(r) => r.status(),
            Err(e) => e.as_response_error().status_code(),
        }
    }};
}

// ─── Slice 4B: auth middleware ────────────────────────────────────────────────

#[cfg(test)]
mod auth_middleware {
    use super::*;
    use actix_web::http::StatusCode;

    /// BLC-AUTH-001: missing Authorization header returns 401.
    #[actix_web::test]
    async fn blc_auth_001_no_auth_header_returns_401() {
        let db = test_db().await.unwrap();
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get().uri("/api/v1/songs");
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }

    /// BLC-AUTH-001: Authorization: Basic abc (wrong scheme) returns 401.
    #[actix_web::test]
    async fn blc_auth_001_basic_scheme_returns_401() {
        let db = test_db().await.unwrap();
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", "Basic abc"));
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }

    /// BLC-AUTH-001: empty Authorization header value returns 401.
    #[actix_web::test]
    async fn blc_auth_001_empty_auth_header_returns_401() {
        let db = test_db().await.unwrap();
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", ""));
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }

    /// BLC-AUTH-002: completely invalid Bearer token returns 401.
    #[actix_web::test]
    async fn blc_auth_002_invalid_bearer_token_returns_401() {
        let db = test_db().await.unwrap();
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", "Bearer totallyinvalidtoken"));
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }

    /// BLC-AUTH-002: deleted session ID returns 401.
    #[actix_web::test]
    async fn blc_auth_002_deleted_session_returns_401() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "auth-deleted@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        // Delete the session before using it.
        session_service(&db).delete_session(&token).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }

    /// BLC-AUTH-001: valid Bearer token is accepted (passes through to resource, not 401).
    #[actix_web::test]
    async fn blc_auth_001_valid_bearer_token_passes_auth() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "auth-valid@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_ne!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }
}

// ─── Slice 4C: OpenAPI endpoint ───────────────────────────────────────────────

#[cfg(test)]
mod openapi_endpoint {
    use super::*;
    use actix_web::http::StatusCode;

    /// BLC-DOCS-001: GET /api/docs/openapi.json without auth returns 200 and valid JSON.
    #[actix_web::test]
    async fn blc_docs_001_openapi_without_auth_returns_200() {
        let db = test_db().await.unwrap();
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/docs/openapi.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let _parsed: serde_json::Value =
            serde_json::from_slice(&body).expect("response is valid JSON");
    }

    /// BLC-DOCS-001: GET /api/docs/openapi.json with auth header still returns 200.
    #[actix_web::test]
    async fn blc_docs_001_openapi_with_auth_still_200() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "docs-auth@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/docs/openapi.json")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-DOCS-001: GET /api/v1/docs/openapi.json (wrong prefix) returns 404.
    #[actix_web::test]
    async fn blc_docs_001_wrong_path_returns_404() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "docs-wrong@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/docs/openapi.json")
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::NOT_FOUND);
    }
}

// ─── Slice 4D: HTTP contract ──────────────────────────────────────────────────

#[cfg(test)]
mod http_contract {
    use super::*;
    use actix_web::http::StatusCode;

    async fn authed_token(db: &Arc<Database>, email: &str) -> String {
        let user = create_user(db, email).await.unwrap();
        create_session_token(db, user).await.unwrap()
    }

    /// BLC-HTTP-001: wrong-table prefix in song ID returns 400.
    #[actix_web::test]
    async fn blc_http_001_wrong_table_prefix_song_returns_400() {
        let db = test_db().await.unwrap();
        let token = authed_token(&db, "http-contract-a@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs/blob:wrongtable")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-HTTP-001: wrong-table prefix in setlist ID returns 400.
    #[actix_web::test]
    async fn blc_http_001_wrong_table_prefix_setlist_returns_400() {
        let db = test_db().await.unwrap();
        let token = authed_token(&db, "http-contract-b@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::delete()
            .uri("/api/v1/setlists/collection:x")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-HTTP-001: correct table prefix is accepted (not 400).
    #[actix_web::test]
    async fn blc_http_001_correct_table_prefix_not_400() {
        let db = test_db().await.unwrap();
        let token = authed_token(&db, "http-contract-c@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs/song:validid")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-HTTP-001: plain ID (no table prefix) is accepted (not 400).
    #[actix_web::test]
    async fn blc_http_001_plain_id_not_400() {
        let db = test_db().await.unwrap();
        let token = authed_token(&db, "http-contract-d@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs/plainid")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-HTTP-002: DELETE existing song, repeat DELETE returns 404.
    #[actix_web::test]
    async fn blc_http_002_idempotent_delete_second_returns_404() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "http-del@test.local").await.unwrap();
        let token = create_session_token(&db, user.clone()).await.unwrap();
        let song = create_song_with_title(&db, &user, "DeleteMe")
            .await
            .unwrap();
        let song_id = song.id.clone();

        let app = test::init_service(build_app(db.clone())).await;

        let first = test::TestRequest::delete()
            .uri(&format!("/api/v1/songs/{song_id}"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let first_resp = test::call_service(&app, first).await;
        assert_eq!(first_resp.status(), StatusCode::OK);

        let second = test::TestRequest::delete()
            .uri(&format!("/api/v1/songs/{song_id}"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let second_resp = test::call_service(&app, second).await;
        assert_eq!(second_resp.status(), StatusCode::NOT_FOUND);
    }

    /// BLC-HTTP-002: DELETE non-existent song ID returns 404.
    #[actix_web::test]
    async fn blc_http_002_delete_nonexistent_returns_404() {
        let db = test_db().await.unwrap();
        let token = authed_token(&db, "http-del-ne@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::delete()
            .uri("/api/v1/songs/nonexistentsongid123")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

// ─── Slice 4E: user admin gates ───────────────────────────────────────────────

#[cfg(test)]
mod user_admin_gates {
    use super::*;
    use actix_web::http::StatusCode;
    use serde_json::json;

    async fn make_admin(db: &Arc<Database>, email: &str) -> (User, String) {
        use crate::test_helpers::user_service;
        use shared::user::Role;
        let mut raw = User::new(email);
        raw.role = Role::Admin;
        let admin = user_service(db).create_user(raw).await.unwrap();
        let token = create_session_token(db, admin.clone()).await.unwrap();
        (admin, token)
    }

    /// BLC-USER-005: authenticated GET /users/me returns 200 matching the user.
    #[actix_web::test]
    async fn blc_user_005_get_me_returns_own_user() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "me-user@test.local").await.unwrap();
        let token = create_session_token(&db, user.clone()).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users/me")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], user.id);
        assert_eq!(body["email"], user.email);
    }

    /// BLC-USER-005: two different users each see their own record via /users/me.
    #[actix_web::test]
    async fn blc_user_005_different_users_see_own_record() {
        let db = test_db().await.unwrap();
        let user_a = create_user(&db, "me-a@test.local").await.unwrap();
        let user_b = create_user(&db, "me-b@test.local").await.unwrap();
        let token_a = create_session_token(&db, user_a.clone()).await.unwrap();
        let token_b = create_session_token(&db, user_b.clone()).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;

        let req_a = test::TestRequest::get()
            .uri("/api/v1/users/me")
            .insert_header(("Authorization", format!("Bearer {token_a}")))
            .to_request();
        let resp_a: serde_json::Value = test::call_and_read_body_json(&app, req_a).await;
        assert_eq!(resp_a["id"], user_a.id);

        let req_b = test::TestRequest::get()
            .uri("/api/v1/users/me")
            .insert_header(("Authorization", format!("Bearer {token_b}")))
            .to_request();
        let resp_b: serde_json::Value = test::call_and_read_body_json(&app, req_b).await;
        assert_eq!(resp_b["id"], user_b.id);
    }

    /// BLC-USER-006: raw token (no Bearer prefix) on GET /users/me returns 200.
    /// The `authorization_bearer` function accepts raw tokens (no scheme) on all routes.
    #[actix_web::test]
    async fn blc_user_006_raw_token_on_me_returns_200() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "raw-token@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users/me")
            .insert_header(("Authorization", token.clone()));
        assert_eq!(call_status!(app, req), StatusCode::OK);
    }

    /// BLC-USER-007: non-admin GET /users returns 403.
    #[actix_web::test]
    async fn blc_user_007_non_admin_get_users_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-lu@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users")
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-USER-007: non-admin POST /users returns 403.
    #[actix_web::test]
    async fn blc_user_007_non_admin_post_users_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-cu@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/users")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json!({"email": "new@test.local"}).to_string());
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-USER-007: non-admin DELETE /users/{id} returns 403.
    #[actix_web::test]
    async fn blc_user_007_non_admin_delete_user_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-du@test.local").await.unwrap();
        let other = create_user(&db, "nonadmin-du-other@test.local")
            .await
            .unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/users/{}", other.id))
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-USER-007: non-admin GET /users/{id} returns 403.
    #[actix_web::test]
    async fn blc_user_007_non_admin_get_user_by_id_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-gu@test.local").await.unwrap();
        let other = create_user(&db, "nonadmin-gu-other@test.local")
            .await
            .unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/{}", other.id))
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-USER-007: admin GET /users returns 200.
    #[actix_web::test]
    async fn blc_user_007_admin_get_users_returns_200() {
        let db = test_db().await.unwrap();
        let (_, token) = make_admin(&db, "admin-lu@test.local").await;

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-USER-009: admin GET /users/{id} returns 200.
    #[actix_web::test]
    async fn blc_user_009_admin_get_user_by_id_returns_200() {
        let db = test_db().await.unwrap();
        let (_, token) = make_admin(&db, "admin-gu@test.local").await;
        let other = create_user(&db, "target-user@test.local").await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/{}", other.id))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-USER-007: admin POST /users with valid email returns 201.
    #[actix_web::test]
    async fn blc_user_007_admin_create_user_returns_201() {
        let db = test_db().await.unwrap();
        let (_, token) = make_admin(&db, "admin-post@test.local").await;

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/users")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(json!({"email": "newly-created@test.local"}).to_string())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}

// ─── Slice 4F: session admin gates ───────────────────────────────────────────

#[cfg(test)]
mod session_admin_gates {
    use super::*;
    use actix_web::http::StatusCode;

    async fn make_admin(db: &Arc<Database>, email: &str) -> (User, String) {
        use crate::test_helpers::user_service;
        use shared::user::Role;
        let mut raw = User::new(email);
        raw.role = Role::Admin;
        let admin = user_service(db).create_user(raw).await.unwrap();
        let token = create_session_token(db, admin.clone()).await.unwrap();
        (admin, token)
    }

    /// BLC-SESS-003: GET /users/me/sessions returns only own sessions.
    ///
    /// User A has 2 sessions; they should see exactly 2.
    #[actix_web::test]
    async fn blc_sess_003_get_my_sessions_returns_own_sessions() {
        let db = test_db().await.unwrap();
        let user_a = create_user(&db, "sess-a@test.local").await.unwrap();
        let user_b = create_user(&db, "sess-b@test.local").await.unwrap();

        let token_a1 = create_session_token(&db, user_a.clone()).await.unwrap();
        let _token_a2 = create_session_token(&db, user_a.clone()).await.unwrap();
        let _token_b1 = create_session_token(&db, user_b.clone()).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users/me/sessions")
            .insert_header(("Authorization", format!("Bearer {token_a1}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        let sessions = body.as_array().expect("array");
        assert_eq!(sessions.len(), 2, "User A should see exactly 2 sessions");
    }

    /// BLC-SESS-003: user with a single session sees exactly one entry.
    #[actix_web::test]
    async fn blc_sess_003_single_session_returns_one_entry() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "sess-one@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users/me/sessions")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body.as_array().expect("array").len(), 1);
    }

    /// BLC-SESS-004: DELETE /users/me/sessions/{own_id} succeeds.
    #[actix_web::test]
    async fn blc_sess_004_delete_own_session_succeeds() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "sess-del-own@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/users/me/sessions/{token}"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-SESS-004: GET /users/me/sessions/{other_user_session} should return 404.
    ///
    #[actix_web::test]
    async fn blc_sess_004_get_other_users_session_via_me_returns_404() {
        let db = test_db().await.unwrap();
        let user_a = create_user(&db, "sess-scope-a@test.local").await.unwrap();
        let user_b = create_user(&db, "sess-scope-b@test.local").await.unwrap();
        let token_a = create_session_token(&db, user_a).await.unwrap();
        let token_b = create_session_token(&db, user_b).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/me/sessions/{token_b}"))
            .insert_header(("Authorization", format!("Bearer {token_a}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// BLC-SESS-005: non-admin GET /users/{other}/sessions returns 403.
    #[actix_web::test]
    async fn blc_sess_005_non_admin_get_other_sessions_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-gs@test.local").await.unwrap();
        let other = create_user(&db, "nonadmin-gs-other@test.local")
            .await
            .unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/{}/sessions", other.id))
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-SESS-005: non-admin POST /users/{other}/sessions returns 403.
    #[actix_web::test]
    async fn blc_sess_005_non_admin_create_other_session_returns_403() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "nonadmin-cs@test.local").await.unwrap();
        let other = create_user(&db, "nonadmin-cs-other@test.local")
            .await
            .unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/users/{}/sessions", other.id))
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::FORBIDDEN);
    }

    /// BLC-SESS-006: admin GET /users/{other}/sessions returns 200.
    #[actix_web::test]
    async fn blc_sess_006_admin_get_other_sessions_returns_200() {
        let db = test_db().await.unwrap();
        let (_, admin_token) = make_admin(&db, "admin-gs@test.local").await;
        let other = create_user(&db, "admin-gs-other@test.local").await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/{}/sessions", other.id))
            .insert_header(("Authorization", format!("Bearer {admin_token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-SESS-006: admin POST /users/{user_id}/sessions returns 201.
    #[actix_web::test]
    async fn blc_sess_006_admin_create_session_for_user_returns_201() {
        let db = test_db().await.unwrap();
        let (_, admin_token) = make_admin(&db, "admin-cs@test.local").await;
        let other = create_user(&db, "admin-cs-other@test.local").await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/users/{}/sessions", other.id))
            .insert_header(("Authorization", format!("Bearer {admin_token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    /// BLC-SESS-006: admin DELETE /users/{user_id}/sessions/{id} returns 200.
    #[actix_web::test]
    async fn blc_sess_006_admin_delete_other_session_returns_200() {
        let db = test_db().await.unwrap();
        let (_, admin_token) = make_admin(&db, "admin-ds@test.local").await;
        let other = create_user(&db, "admin-ds-other@test.local").await.unwrap();
        let other_token = create_session_token(&db, other.clone()).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::delete()
            .uri(&format!(
                "/api/v1/users/{}/sessions/{}",
                other.id, other_token
            ))
            .insert_header(("Authorization", format!("Bearer {admin_token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-SESS-009: deleted session token on an authenticated route returns 401.
    #[actix_web::test]
    async fn blc_sess_009_deleted_session_token_returns_401() {
        let db = test_db().await.unwrap();
        let user = create_user(&db, "sess-del@test.local").await.unwrap();
        let token = create_session_token(&db, user).await.unwrap();

        session_service(&db).delete_session(&token).await.unwrap();

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", format!("Bearer {token}")));
        assert_eq!(call_status!(app, req), StatusCode::UNAUTHORIZED);
    }
}

// ─── Slice 4G: list pagination HTTP validation ───────────────────────────────

#[cfg(test)]
mod list_pagination {
    use super::*;
    use actix_web::http::StatusCode;

    async fn authed_user_and_token(db: &Arc<Database>, email: &str) -> (User, String) {
        let user = create_user(db, email).await.unwrap();
        let token = create_session_token(db, user.clone()).await.unwrap();
        (user, token)
    }

    /// BLC-LP-004: non-integer `page` query param returns 400.
    #[actix_web::test]
    async fn blc_lp_004_non_integer_page_returns_400() {
        let db = test_db().await.unwrap();
        let (_, token) = authed_user_and_token(&db, "lp004a@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page=abc")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-LP-004: non-integer `page_size` returns 400.
    #[actix_web::test]
    async fn blc_lp_004_non_integer_page_size_returns_400() {
        let db = test_db().await.unwrap();
        let (_, token) = authed_user_and_token(&db, "lp004b@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page_size=1.5")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// BLC-LP-004: valid integer page and page_size returns 200.
    #[actix_web::test]
    async fn blc_lp_004_valid_page_and_page_size_returns_200() {
        let db = test_db().await.unwrap();
        let (_, token) = authed_user_and_token(&db, "lp004c@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page=0&page_size=10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-LP-005: whitespace-only `q` treated same as absent (same result count as no q).
    #[actix_web::test]
    async fn blc_lp_005_whitespace_q_treated_as_absent() {
        let db = test_db().await.unwrap();
        let (user, token) = authed_user_and_token(&db, "lp005a@test.local").await;
        create_song_with_title(&db, &user, "Whitespace Test Song")
            .await
            .unwrap();

        let app = test::init_service(build_app(db.clone())).await;

        let no_q = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let no_q_resp: serde_json::Value = test::call_and_read_body_json(&app, no_q).await;

        let ws_q = test::TestRequest::get()
            .uri("/api/v1/songs?q=%20%20")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let ws_q_resp: serde_json::Value = test::call_and_read_body_json(&app, ws_q).await;

        assert_eq!(
            no_q_resp.as_array().expect("array").len(),
            ws_q_resp.as_array().expect("array").len(),
            "whitespace q should return same count as no q"
        );
    }

    /// BLC-LP-005: empty `q` treated same as absent.
    #[actix_web::test]
    async fn blc_lp_005_empty_q_treated_as_absent() {
        let db = test_db().await.unwrap();
        let (user, token) = authed_user_and_token(&db, "lp005b@test.local").await;
        create_song_with_title(&db, &user, "Empty Q Song")
            .await
            .unwrap();

        let app = test::init_service(build_app(db.clone())).await;

        let no_q = test::TestRequest::get()
            .uri("/api/v1/songs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let no_q_resp: serde_json::Value = test::call_and_read_body_json(&app, no_q).await;

        let empty_q = test::TestRequest::get()
            .uri("/api/v1/songs?q=")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let empty_q_resp: serde_json::Value = test::call_and_read_body_json(&app, empty_q).await;

        assert_eq!(
            no_q_resp.as_array().unwrap().len(),
            empty_q_resp.as_array().unwrap().len()
        );
    }

    /// BLC-LP-006: page_size=0 returns all items.
    #[actix_web::test]
    async fn blc_lp_006_page_size_zero_returns_all() {
        let db = test_db().await.unwrap();
        let (user, token) = authed_user_and_token(&db, "lp006@test.local").await;
        for i in 0..3 {
            create_song_with_title(&db, &user, &format!("LP006 Song {i}"))
                .await
                .unwrap();
        }

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page_size=0")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.as_array().unwrap().len(), 3);
    }

    /// BLC-LP-007: only `page` supplied (no page_size) returns 200.
    #[actix_web::test]
    async fn blc_lp_007_only_page_returns_200() {
        let db = test_db().await.unwrap();
        let (_, token) = authed_user_and_token(&db, "lp007a@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page=0")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-LP-007: only `page_size` supplied (no page) returns 200.
    #[actix_web::test]
    async fn blc_lp_007_only_page_size_returns_200() {
        let db = test_db().await.unwrap();
        let (_, token) = authed_user_and_token(&db, "lp007b@test.local").await;
        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page_size=5")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// BLC-LP-008: page beyond last returns 200 with empty array.
    #[actix_web::test]
    async fn blc_lp_008_page_beyond_last_returns_empty() {
        let db = test_db().await.unwrap();
        let (user, token) = authed_user_and_token(&db, "lp008@test.local").await;
        for i in 0..3 {
            create_song_with_title(&db, &user, &format!("LP008 Song {i}"))
                .await
                .unwrap();
        }

        let app = test::init_service(build_app(db.clone())).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/songs?page=999&page_size=10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.as_array().unwrap().is_empty());
    }

    /// BLC-LP-009: q filter applies before pagination — paginated matching subset returned.
    ///
    /// 3 songs match `q=matchtoken`; with `page_size=2&page=0` → 2 results,
    /// with `page_size=2&page=1` → 1 result.
    #[actix_web::test]
    async fn blc_lp_009_q_filter_applies_before_pagination() {
        let db = test_db().await.unwrap();
        let (user, token) = authed_user_and_token(&db, "lp009@test.local").await;

        for i in 0..3 {
            create_song_with_title(&db, &user, &format!("matchtoken LP009 Song {i}"))
                .await
                .unwrap();
        }
        for i in 0..2 {
            create_song_with_title(&db, &user, &format!("Other LP009 Song {i}"))
                .await
                .unwrap();
        }

        let app = test::init_service(build_app(db.clone())).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/songs?q=matchtoken&page=0&page_size=2")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(
            resp.as_array().unwrap().len(),
            2,
            "page 0 of 3 matching with page_size=2 should return 2"
        );

        let req2 = test::TestRequest::get()
            .uri("/api/v1/songs?q=matchtoken&page=1&page_size=2")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp2: serde_json::Value = test::call_and_read_body_json(&app, req2).await;
        assert_eq!(
            resp2.as_array().unwrap().len(),
            1,
            "page 1 of 3 matching with page_size=2 should return 1"
        );
    }
}
