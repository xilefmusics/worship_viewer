mod error;
mod settings;
mod types;

use error::AppError;
use settings::Settings;
use types::{Group, IdWrapper};

use actix_web::{post, web::Data, web::Json, App, HttpRequest, HttpResponse, HttpServer};
use env_logger::Env;
use futures::future::join_all;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub fn parse_user_header(req: HttpRequest) -> Result<String, AppError> {
    Ok(req
        .headers()
        .get("X-Remote-User")
        .ok_or(AppError::Unauthorized("no X-Remote-User given".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("no X-Remote-User given".into()))?
        .into())
}

pub fn exspect_admin(user: &str) -> Result<(), AppError> {
    // TODO: Check if the user has admin rights
    if user == "admin" {
        Ok(())
    } else {
        Err(AppError::Unauthorized(
            "user doesn't have admin rights".into(),
        ))
    }
}

pub async fn create_group(
    db: &Data<Surreal<Client>>,
    wrapper: &IdWrapper<Group>,
) -> Result<Group, AppError> {
    db.create(("group", wrapper.id.clone()))
        .content(wrapper.data.clone())
        .await
        .map_err(|err| AppError::Database(format!("{}", err)))?
        .ok_or(AppError::Database("record is none".into()))
}

pub async fn create_groups(
    db: &Data<Surreal<Client>>,
    groups: &Vec<IdWrapper<Group>>,
) -> Result<Vec<Group>, AppError> {
    let tmp1 = groups.into_iter().map(|group| create_group(db, &group));
    join_all(tmp1).await.into_iter().collect()
}

#[post("/api/groups")]
pub async fn post_groups(
    req: HttpRequest,
    groups: Json<Vec<IdWrapper<Group>>>,
    db: Data<Surreal<Client>>,
) -> Result<HttpResponse, AppError> {
    exspect_admin(&parse_user_header(req)?)?;
    Ok(HttpResponse::Ok().json(create_groups(&db, &groups).await?))
}

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let settings = Settings::new();

    let database = Surreal::new::<Ws>(format!("{}:{}", settings.db_host, settings.db_port))
        .await
        .unwrap();
    database
        .signin(Root {
            username: &settings.db_user,
            password: &settings.db_password,
        })
        .await
        .unwrap();
    database
        .use_ns(settings.db_namespace)
        .use_db(settings.db_database)
        .await
        .unwrap();
    let database = Data::new(database);

    HttpServer::new(move || {
        let database = database.clone();
        App::new().app_data(database).service(post_groups)
    })
    .bind((settings.host, settings.port))
    .unwrap()
    .run()
    .await
    .unwrap()
}
