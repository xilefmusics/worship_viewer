use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result as AnyResult};
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl};

use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum OidcProvider {
    Google,
    Apple,
}

impl OidcProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Apple => "apple",
        }
    }

    fn issuer_error_message(&self) -> &'static str {
        match self {
            Self::Google => "invalid GOOGLE_ISSUER_URL value",
            Self::Apple => "invalid APPLE_ISSUER_URL value",
        }
    }

    fn metadata_error_message(&self) -> &'static str {
        match self {
            Self::Google => "unable to fetch Google provider metadata",
            Self::Apple => "unable to fetch Apple provider metadata",
        }
    }

    fn redirect_error_message(&self) -> &'static str {
        match self {
            Self::Google => "invalid GOOGLE_REDIRECT_URL value",
            Self::Apple => "invalid APPLE_REDIRECT_URL value",
        }
    }
}

impl fmt::Display for OidcProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for OidcProvider {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "google" => Ok(Self::Google),
            "apple" => Ok(Self::Apple),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct OidcClientRegistration {
    client: Arc<CoreClient>,
    scopes: Vec<String>,
}

impl OidcClientRegistration {
    pub fn client(&self) -> &CoreClient {
        self.client.as_ref()
    }

    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
}

#[derive(Debug)]
pub struct OidcClients {
    default_provider: OidcProvider,
    registrations: HashMap<OidcProvider, OidcClientRegistration>,
}

impl OidcClients {
    pub fn get(&self, provider: &OidcProvider) -> Option<&OidcClientRegistration> {
        self.registrations.get(provider)
    }

    pub fn default_provider(&self) -> OidcProvider {
        self.default_provider
    }
}

pub async fn build_clients(settings: &Settings) -> AnyResult<OidcClients> {
    let mut registrations = HashMap::new();

    let google_client = build_client(
        OidcProvider::Google,
        &settings.oidc_issuer_url,
        &settings.oidc_client_id,
        settings.oidc_client_secret.as_deref(),
        &settings.oidc_redirect_url,
    )
    .await?;

    registrations.insert(
        OidcProvider::Google,
        OidcClientRegistration {
            client: Arc::new(google_client),
            scopes: settings.oidc_scopes.clone(),
        },
    );

    if let Some(client_id) = settings
        .apple_client_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        let issuer_url = settings
            .apple_issuer_url
            .clone()
            .unwrap_or_else(|| "https://appleid.apple.com".to_string());
        let redirect_url = settings
            .apple_redirect_url
            .clone()
            .unwrap_or_else(|| settings.oidc_redirect_url.clone());

        let apple_client = build_client(
            OidcProvider::Apple,
            &issuer_url,
            client_id,
            settings.apple_client_secret.as_deref(),
            &redirect_url,
        )
        .await?;

        registrations.insert(
            OidcProvider::Apple,
            OidcClientRegistration {
                client: Arc::new(apple_client),
                scopes: settings
                    .apple_scopes
                    .clone()
                    .unwrap_or_else(|| vec!["openid".into(), "email".into(), "name".into()]),
            },
        );
    }

    Ok(OidcClients {
        default_provider: OidcProvider::Google,
        registrations,
    })
}

async fn build_client(
    provider: OidcProvider,
    issuer_url: &str,
    client_id: &str,
    client_secret: Option<&str>,
    redirect_url: &str,
) -> AnyResult<CoreClient> {
    let issuer =
        IssuerUrl::new(issuer_url.to_string()).with_context(|| provider.issuer_error_message())?;
    let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
        .await
        .with_context(|| provider.metadata_error_message())?;
    let redirect = RedirectUrl::new(redirect_url.to_string())
        .with_context(|| provider.redirect_error_message())?;

    Ok(CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(client_id.to_string()),
        client_secret.map(|secret| ClientSecret::new(secret.to_owned())),
    )
    .set_redirect_uri(redirect))
}
