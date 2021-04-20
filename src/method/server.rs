use rocket_contrib::json::Json;

use std::env;
use std::path::PathBuf;

use super::super::line::IterExtTranspose;
use super::super::song::Song;
use super::Error;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/song/<title>/<key>")]
fn get_song(title: String, key: String) -> Result<String, ()> {
    let path = PathBuf::from("/home/xilef/Songs");
    Song::load_all(&path)
        .map_err(|_| ())?
        .into_iter()
        .find(|song| song.title == title)
        .ok_or(())?
        .load_lines()
        .map_err(|_| ())?
        .into_iter()
        .transpose(&key);
    Ok(format!("Not yet implemented! ({}, {})", title, key))
}

#[get("/titles")]
fn get_titles() -> Result<Json<Vec<String>>, ()> {
    let path = PathBuf::from("/home/xilef/Songs");
    Ok(Json(
        Song::load_all(&path)
            .map_err(|_| ())?
            .into_iter()
            .map(|song| song.title)
            .collect(),
    ))
}

pub fn server(_args: env::Args) -> Result<(), Error> {
    rocket::ignite()
        .mount("/", routes![index, get_song, get_titles])
        .launch();
    Ok(())
}
