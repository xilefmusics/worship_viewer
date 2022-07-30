use std::env;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use crate::server::{rest_api, ws_broadcaster};
use crate::setlist::SetlistPool;
use crate::song::SongPool;

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

pub fn server(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;

    let song_pool = Arc::new(SongPool::new_local(config.song_path.clone())?);
    let setlist_pool = Arc::new(SetlistPool::new_local(
        config.setlist_path.clone(),
        Arc::clone(&song_pool),
    )?);

    // websocket broadcaster
    thread::spawn(|| -> Result<(), Error> { Ok(ws_broadcaster(Ipv4Addr::UNSPECIFIED, 8001)?) });

    rest_api(
        song_pool,
        setlist_pool,
        config.web_path,
        Ipv4Addr::UNSPECIFIED,
        8000,
    );
    Ok(())
}
