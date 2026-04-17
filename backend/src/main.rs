use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use anyhow::{Context, Result as AnyResult};
use chrono::Utc;
use lettre::transport::smtp::authentication::Credentials;
use tracing::info;
use tracing_subscriber::EnvFilter;

use backend::auth;
use backend::auth::oidc;
use backend::database;
use backend::docs;
use backend::frontend;
use backend::mail::MailService;
use backend::resources;
use backend::resources::Session;
use backend::resources::blob::service::BlobServiceHandle;
use backend::resources::collection::service::CollectionServiceHandle;
use backend::resources::setlist::{SetlistService, SurrealSetlistRepo};
use backend::resources::song::service::SongServiceHandle;
use backend::resources::team::invitation::InvitationServiceHandle;
use backend::resources::team::{SurrealTeamResolver, TeamServiceHandle};
use backend::resources::user::service::UserServiceHandle;
use backend::resources::user::session::service::SessionServiceHandle;
use backend::resources::user::{Role as UserRole, User};
use backend::settings::Settings;

#[actix_web::main]
async fn main() -> AnyResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let settings = Settings::from_env()?;

    let cookie_config = Data::new(settings.cookie_config());
    let otp_config = Data::new(settings.otp_config());

    let mail_service = MailService::new(
        settings.gmail_from.clone(),
        Credentials::new(
            settings.gmail_from.clone(),
            settings.gmail_app_password.clone(),
        ),
    )?;

    let db = Arc::new(
        database::Database::connect(
            &settings.db_address,
            &settings.db_namespace,
            &settings.db_database,
            settings.db_username.as_deref(),
            settings.db_password.as_deref(),
        )
        .await?,
    );
    db.migrate(settings.db_migration_path.as_str())
        .await
        .context("database migration failed")?;

    let user_service = UserServiceHandle::build(db.clone());
    let session_service = SessionServiceHandle::build(db.clone());

    if let Some(email) = settings.initial_admin_user_email.as_ref() {
        let (admin, created_initial_admin) = if let Some(user) = user_service
            .get_user_by_email(email)
            .await
            .context("failed to look up initial admin user by email")?
        {
            info!(
                "Initial admin user already exists ({}), not creating: {}",
                user.id, user.email
            );
            (user, false)
        } else {
            let admin = user_service
                .create_user(User {
                    id: String::new(),
                    email: email.to_owned(),
                    role: UserRole::Admin,
                    default_collection: None,
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
            (admin, true)
        };

        if settings.initial_admin_user_test_session {
            if created_initial_admin {
                let session = session_service
                    .create_session(Session::admin(admin, settings.session_ttl_seconds as i64))
                    .await
                    .context("failed to create a test session for the admin user")?;
                info!(
                    "Created a test session {} for the admin user. DO NOT USE THIS IN PRODUCTION",
                    session.id,
                );
            } else {
                info!("Initial admin user was not created on this run, not creating test session");
            }
        }
    }

    let oidc_clients = Data::new(Arc::new(oidc::build_clients(&settings).await?));

    let blob_service = BlobServiceHandle::build(db.clone(), settings.blob_dir.clone());
    let collection_service = CollectionServiceHandle::build(db.clone());
    let song_service = SongServiceHandle::build(db.clone());
    let setlist_service = SetlistService::new(
        SurrealSetlistRepo::new(db.clone()),
        SurrealTeamResolver::new(db.clone()),
        db.clone(),
    );
    let team_service = TeamServiceHandle::build(db.clone());
    let invitation_service = InvitationServiceHandle::build(db.clone());
    let static_dir = settings.static_dir.clone();
    let db_data = Data::from(db);

    info!(
        "Starting server on http://{}:{}",
        settings.host, settings.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(Data::new(mail_service.clone()))
            .app_data(Data::new(blob_service.clone()))
            .app_data(Data::new(collection_service.clone()))
            .app_data(Data::new(song_service.clone()))
            .app_data(Data::new(setlist_service.clone()))
            .app_data(Data::new(team_service.clone()))
            .app_data(Data::new(invitation_service.clone()))
            .app_data(Data::new(user_service.clone()))
            .app_data(Data::new(session_service.clone()))
            .app_data(oidc_clients.clone())
            .app_data(cookie_config.clone())
            .app_data(otp_config.clone())
            .wrap(Logger::default())
            .service(auth::rest::scope(
                settings.auth_rate_limit_rps,
                settings.auth_rate_limit_burst,
            ))
            .service(docs::rest::scope())
            .service(resources::rest::scope(settings.blob_upload_max_bytes))
            .service(frontend::rest::scope(&static_dir))
    })
    .bind((settings.host.clone(), settings.port))?
    .run()
    .await
    .context("server exited unexpectedly")
}
