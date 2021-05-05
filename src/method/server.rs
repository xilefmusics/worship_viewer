use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use ws::listen;

use std::env;
use std::path::PathBuf;
use std::thread;

use crate::setlist::SetlistItem;
use crate::song::{Song, SongPool};

use super::Error;

pub struct Config {
    pub path: PathBuf,
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
        let path = PathBuf::from(path.unwrap_or(".".to_string()));
        let web_path = match web_path {
            Some(web_path) => PathBuf::from(web_path),
            None => path.join("www"),
        };
        Ok(Self { path, web_path })
    }
}

pub struct MyState {
    pub config: Config,
    pub song_pool: SongPool,
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

#[get("/titles")]
fn get_titles(state: State<MyState>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(state.song_pool.titles().map_err(|_| ())?))
}

pub fn server(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;
    let song_pool = SongPool::new_local(&config.path)?;
    let state = MyState { config, song_pool };

    // websocket broadcaster
    thread::spawn(|| {
        listen("0.0.0.0:8001", |out| move |msg| out.broadcast(msg)).unwrap();
    });

    // REST API
    rocket::ignite()
        .mount(
            "/",
            routes![index, get_song, get_titles, get_song_without_key],
        )
        .mount(
            "/static",
            StaticFiles::from(state.config.web_path.join("static")),
        )
        .manage(state)
        .launch();
    Ok(())
}
