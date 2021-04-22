use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use ws::listen;

use std::env;
use std::path::PathBuf;
use std::thread;

use crate::song::{SectionSong, Song};

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

#[get("/")]
fn index(config: State<Config>) -> Option<NamedFile> {
    NamedFile::open(config.web_path.join("index.html")).ok()
}

#[get("/song/<title>/<key>")]
fn get_song(title: String, key: String, config: State<Config>) -> Option<Json<SectionSong>> {
    let song = Song::load_all(&config.path)
        .ok()?
        .into_iter()
        .find(|song| song.title == title)?;
    Some(Json(song.load_section_song(&key).ok()?))
}

#[get("/song/<title>")]
fn get_song_without_key(title: String, config: State<Config>) -> Option<Json<SectionSong>> {
    get_song(title, "Self".to_string(), config)
}

#[get("/titles")]
fn get_titles(config: State<Config>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(
        Song::load_all(&config.path)
            .map_err(|_| ())?
            .into_iter()
            .map(|song| song.title)
            .collect(),
    ))
}

pub fn server(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;

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
        .mount("/static", StaticFiles::from(config.web_path.join("static")))
        .manage(config)
        .launch();
    Ok(())
}
