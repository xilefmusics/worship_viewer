mod database;
mod error;
mod rest;
mod settings;
mod types;

use database::Database;
use error::AppError;
use rest::{get, post};
use settings::Settings;

use actix_web::{web::Data, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let settings = Settings::new();
    let database = Data::new(Database::new(settings.clone()).await);

    HttpServer::new(move || {
        let database = database.clone();
        App::new()
            .app_data(database)
            .service(get::groups)
            .service(post::groups)
            .service(get::users)
            .service(post::users)
            .service(get::blobs_metadata)
            .service(post::blobs_metadata)
            .service(get::songs)
            .service(post::songs)
            .service(get::collections)
            .service(post::collections)
    })
    .bind((settings.host, settings.port))
    .unwrap()
    .run()
    .await
    .unwrap()
}
