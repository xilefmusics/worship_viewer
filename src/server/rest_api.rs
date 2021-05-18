use rocket::config::{Config, Environment};
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use crate::setlist::{Setlist, SetlistItem, SetlistPool};
use crate::song::{Song, SongPool};

struct MyState {
    pub static_path: PathBuf,
    pub song_pool: Arc<SongPool>,
    pub setlist_pool: Arc<SetlistPool>,
}

#[get("/")]
fn index(state: State<MyState>) -> Option<NamedFile> {
    NamedFile::open(state.static_path.join("index.html")).ok()
}

#[get("/song/<title>/<key>")]
fn get_song(title: String, key: String, state: State<MyState>) -> Result<Option<Json<Song>>, ()> {
    let song = state
        .song_pool
        .get(&SetlistItem { title, key })
        .map_err(|_| ())?;
    match song {
        Some(song) => Ok(Some(Json(song))),
        None => Ok(None),
    }
}

#[get("/song/<title>")]
fn get_song_without_key(title: String, state: State<MyState>) -> Result<Option<Json<Song>>, ()> {
    get_song(title, "Self".to_string(), state)
}

#[get("/song_titles")]
fn get_song_titles(state: State<MyState>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(state.song_pool.titles().map_err(|_| ())?))
}

#[get("/setlist_titles")]
fn get_setlist_titles(state: State<MyState>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(state.setlist_pool.titles().map_err(|_| ())?))
}

#[get("/setlist_true_titles")]
fn get_setlist_true_titles(state: State<MyState>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(state.setlist_pool.true_titles().map_err(|_| ())?))
}

#[get("/setlist_all_songs")]
fn get_setlist_all_songs(state: State<MyState>) -> Result<Json<Setlist>, ()> {
    Ok(Json(state.setlist_pool.all_songs().map_err(|_| ())?))
}

#[get("/setlist/<title>")]
fn get_setlist(title: String, state: State<MyState>) -> Result<Option<Json<Setlist>>, ()> {
    let setlist = state.setlist_pool.get(title).map_err(|_| ())?;
    match setlist {
        Some(setlist) => Ok(Some(Json(setlist))),
        None => Ok(None),
    }
}

#[get("/setlist_get_first")]
fn get_first_setlist(state: State<MyState>) -> Result<Option<Json<Setlist>>, ()> {
    let setlist = state.setlist_pool.get_first().map_err(|_| ())?;
    match setlist {
        Some(setlist) => Ok(Some(Json(setlist))),
        None => Ok(None),
    }
}

pub fn rest_api(
    song_pool: Arc<SongPool>,
    setlist_pool: Arc<SetlistPool>,
    static_path: PathBuf,
    ip: Ipv4Addr,
    port: u16,
) {
    let state = MyState {
        static_path,
        song_pool,
        setlist_pool,
    };

    let config = Config::build(Environment::Staging)
        .address(ip.to_string())
        .port(port)
        .finalize()
        .unwrap();

    // REST API
    rocket::custom(config)
        .mount(
            "/",
            routes![
                index,
                get_song,
                get_song_titles,
                get_song_without_key,
                get_setlist_titles,
                get_setlist_true_titles,
                get_setlist_all_songs,
                get_setlist,
                get_first_setlist,
            ],
        )
        .mount(
            "/static",
            StaticFiles::from(state.static_path.join("static")),
        )
        .manage(state)
        .launch();
}
