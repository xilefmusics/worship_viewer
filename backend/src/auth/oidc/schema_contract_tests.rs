//! SCHEMAFULL contract tests: serde content written by OIDC state helpers must match Surreal schema.

use chrono::{Duration as ChronoDuration, Utc};
use openidconnect::Nonce;
use openidconnect::PkceCodeChallenge;

use super::{Model as OidcModel, OidcProvider, PendingOidc};
use crate::test_helpers::test_db;

#[tokio::test]
async fn oidc_state_remember_and_take_round_trip() {
    let db = test_db().await.expect("test db");

    let (_, verifier) = PkceCodeChallenge::new_random_sha256();
    let now = Utc::now();
    let pending = PendingOidc {
        pkce_verifier: verifier,
        nonce: Nonce::new_random(),
        redirect_to: Some("/after-login".into()),
        created_at: now,
        expires_at: now + ChronoDuration::seconds(600),
        provider: OidcProvider::Google,
    };
    let key = "contract-test-csrf";

    OidcModel::remember_oidc_state(db.as_ref(), key, pending)
        .await
        .expect("remember_oidc_state should accept all OidcStateRecord fields under SCHEMAFULL");

    let taken = OidcModel::take_oidc_state(db.as_ref(), key)
        .await
        .expect("take_oidc_state");
    let Some(out) = taken else {
        panic!("expected stored oidc_state");
    };

    assert_eq!(out.provider, OidcProvider::Google);
    assert_eq!(out.redirect_to.as_deref(), Some("/after-login"));
}
