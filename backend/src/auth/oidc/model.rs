use std::str::FromStr;

use chrono::{DateTime, Utc};
use oauth2::PkceCodeVerifier;
use openidconnect::Nonce;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use crate::database::Database;
use crate::error::AppError;

use super::OidcProvider;

#[derive(Debug)]
pub struct PendingOidc {
    pub pkce_verifier: PkceCodeVerifier,
    pub nonce: Nonce,
    pub redirect_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub provider: OidcProvider,
}

pub trait Model {
    async fn remember_oidc_state(&self, key: &str, value: PendingOidc) -> Result<(), AppError>;
    async fn take_oidc_state(&self, key: &str) -> Result<Option<PendingOidc>, AppError>;
    async fn cleanup_expired_oidc_states(&self) -> Result<(), AppError>;
}

impl Model for Database {
    async fn remember_oidc_state(&self, key: &str, value: PendingOidc) -> Result<(), AppError> {
        let record = OidcStateRecord::new(value);

        let _: Option<OidcStateRecord> = self
            .db
            .create(("oidc_state", key))
            .content(record)
            .await
            .map_err(AppError::database)?;

        Ok(())
    }

    async fn take_oidc_state(&self, key: &str) -> Result<Option<PendingOidc>, AppError> {
        let record: Option<OidcStateRecord> = self
            .db
            .select(("oidc_state", key))
            .await
            .map_err(AppError::database)?;

        let Some(record) = record else {
            return Ok(None);
        };

        let _: Option<OidcStateRecord> = self
            .db
            .delete(("oidc_state", key))
            .await
            .map_err(AppError::database)?;

        let pending = record.into_pending();
        if pending.expires_at <= Utc::now() {
            return Ok(None);
        }

        Ok(Some(pending))
    }

    async fn cleanup_expired_oidc_states(&self) -> Result<(), AppError> {
        self.db
            .query("DELETE oidc_state WHERE expires_at <= time::now()")
            .await
            .map_err(AppError::database)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OidcStateRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    pkce_verifier: String,
    nonce: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    redirect_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    created_at: Datetime,
    expires_at: Datetime,
}

impl OidcStateRecord {
    fn new(pending: PendingOidc) -> Self {
        let PendingOidc {
            pkce_verifier,
            nonce,
            redirect_to,
            created_at,
            expires_at,
            provider,
        } = pending;

        Self {
            id: None,
            pkce_verifier: pkce_verifier.secret().clone(),
            nonce: nonce.secret().to_owned(),
            redirect_to,
            provider: Some(provider.to_string()),
            created_at: created_at.into(),
            expires_at: expires_at.into(),
        }
    }

    fn into_pending(self) -> PendingOidc {
        PendingOidc {
            pkce_verifier: PkceCodeVerifier::new(self.pkce_verifier),
            nonce: Nonce::new(self.nonce),
            redirect_to: self.redirect_to,
            created_at: DateTime::<Utc>::from(self.created_at),
            expires_at: DateTime::<Utc>::from(self.expires_at),
            provider: self
                .provider
                .as_deref()
                .and_then(|value| OidcProvider::from_str(value).ok())
                .unwrap_or(OidcProvider::Google),
        }
    }
}
