use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use ws::listen;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use crate::setlist::{Setlist, SetlistItem, SetlistPool};
use crate::song::{Song, SongPool};

use super::Error;

pub struct Config {
    pub song_path: PathBuf,
    pub setlist_path: PathBuf,
    pub web_path: PathBuf,
}
impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut path: Option<String> = None;
        let mut web_path: Option<String> = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-w" => match args.next() {
                    Some(p) => web_path = Some(p),
                    None => {
                        return Err(Error::ParseArgs("No value for option -w given".to_string()))
                    }
                },
                f => path = Some(f.to_string()),
            }
        }
        let song_path = PathBuf::from(path.unwrap_or(".".to_string()));
        let web_path = match web_path {
            Some(web_path) => PathBuf::from(web_path),
            None => song_path.join("www"),
        };
        let setlist_path = song_path.join("setlists");
        Ok(Self {
            song_path,
            web_path,
            setlist_path,
        })
    }
}

pub struct MyState {
    pub config: Config,
    pub song_pool: Arc<SongPool>,
    pub setlist_pool: Arc<SetlistPool>,
}

#[get("/")]
fn index(state: State<MyState>) -> Option<NamedFile> {
    NamedFile::open(state.config.web_path.join("index.html")).ok()
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

pub fn server(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;
    let song_pool = Arc::new(SongPool::new_local(config.song_path.clone())?);
    let setlist_pool = Arc::new(SetlistPool::new_local(
        config.setlist_path.clone(),
        Arc::clone(&song_pool),
    )?);
    let state = MyState {
        config,
        song_pool,
        setlist_pool,
    };

    // websocket broadcaster
    thread::spawn(|| {
        listen("0.0.0.0:8001", |out| move |msg| out.broadcast(msg)).unwrap();
    });

    // REST API
    rocket::ignite()
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
            StaticFiles::from(state.config.web_path.join("static")),
        )
        .manage(state)
        .launch();
    Ok(())
}
