mod auth;
mod database;
mod docs;
mod error;
mod mail;
mod resources;
mod settings;

use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use anyhow::{Context, Result as AnyResult};
use chrono::Utc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::auth::oidc;
use crate::resources::{Session, SessionModel, User, UserModel, UserRole};
use crate::settings::Settings;

#[actix_web::main]
async fn main() -> AnyResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let settings = Settings::init()?;

    let db = Data::new(database::Database::new().await?);
    db.migrate().await.context("database migration failed")?;

    if let Some(email) = settings.initial_admin_user_email.as_ref() {
        let admin = db
            .create_user(User {
                id: String::new(),
                email: email.to_owned(),
                role: UserRole::Admin,
                read: vec![],
                write: vec![],
                created_at: Utc::now(),
                last_login_at: None,
                request_count: 0,
            })
            .await
            .context("failed to create admin user")?;
        info!(
            "Created admin user {} with email: {}",
            admin.id, admin.email
        );

        if settings.initial_admin_user_test_session {
            let session = db
                .create_session(Session::admin(admin))
                .await
                .context("failed to create a test session for the admin user")?;
            info!(
                "Created a test session {} for the admin user. DO NOT USE THIS IN PRODUCTION",
                session.id,
            );
        }
    }

    let oidc_clients = Data::new(Arc::new(oidc::build_clients(settings).await?));
    info!(
        "Starting server on http://{}:{}",
        settings.host, settings.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .app_data(oidc_clients.clone())
            .wrap(Logger::default())
            .wrap(auth::middleware::cors())
            .service(auth::rest::scope())
            .service(docs::reset::scope())
            .service(resources::rest::scope())
    })
    .bind((settings.host.clone(), settings.port))?
    .run()
    .await
    .context("server exited unexpectedly")
}
