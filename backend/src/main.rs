mod blob;
mod collection;
mod error;
mod import;
mod like;
mod player;
mod rest;
mod settings;
mod song;
mod user;

use actix_files::Files;
use actix_web::{web::Data, web::PayloadConfig, App, HttpServer};
use env_logger::Env;
use error::AppError;
use fancy_surreal::Client;
use settings::Settings;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let settings = Settings::new();
    let database = Data::new(
        Client::new(
            &settings.db_host,
            settings.db_port,
            &settings.db_user,
            &settings.db_password,
            &settings.db_database,
            &settings.db_namespace,
        )
        .await
        .map_err(|err| AppError::Other(format!("Couldn't connect to database ({})", err)))?,
    );

    HttpServer::new(move || {
        let database = database.clone();
        App::new()
            .app_data(database)
            .app_data(PayloadConfig::new(1 << 25))
            .service(user::rest::get)
            .service(user::rest::get_id)
            .service(user::rest::put)
            .service(user::rest::post)
            .service(user::rest::delete)
            .service(blob::rest::get_metadata)
            .service(blob::rest::get_metadata_id)
            .service(blob::rest::put_metadata)
            .service(blob::rest::post_metadata)
            .service(blob::rest::delete_metadata)
            .service(blob::rest::get_id)
            .service(song::rest::get)
            .service(song::rest::get_id)
            .service(song::rest::put)
            .service(song::rest::post)
            .service(song::rest::delete)
            .service(collection::rest::get)
            .service(collection::rest::get_id)
            .service(collection::rest::put)
            .service(collection::rest::post)
            .service(collection::rest::delete)
            .service(like::rest::get)
            .service(like::rest::get_id)
            .service(like::rest::put)
            .service(like::rest::post)
            .service(like::rest::delete)
            .service(like::rest::toggle)
            .service(player::rest::get)
            .service(import::rest::get)
            .service(rest::get_index)
            .service(rest::get_static_files)
            .service(
                Files::new("/", std::env::var("STATIC_DIR").unwrap_or("static".into()))
                    .show_files_listing(),
            )
    })
    .bind((settings.host, settings.port))
    .map_err(|err| AppError::Other(format!("Couldn't bind port ({})", err)))?
    .run()
    .await
    .map_err(|err| AppError::Other(format!("Server crashed ({})", err)))
}
