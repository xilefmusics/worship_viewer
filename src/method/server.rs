use rocket::State;
use rocket_contrib::json::Json;

use std::env;
use std::path::PathBuf;

use crate::song::{SectionSong, Song};

use super::Error;

pub struct Config {
    pub path: PathBuf,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut path: Option<String> = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                f => path = Some(f.to_string()),
            }
        }
        let path = PathBuf::from(path.unwrap_or(".".to_string()));
        Ok(Self { path })
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
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
    rocket::ignite()
        .mount(
            "/",
            routes![index, get_song, get_titles, get_song_without_key],
        )
        .manage(Config::new(args)?)
        .launch();
    Ok(())
}
