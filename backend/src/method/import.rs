use super::Error;
use std::env;

use crate::song::import::ultimate_guitar_to_song;

pub struct Config {
    pub url: String,
}
impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut url: Option<String> = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                u => url = Some(u.to_string()),
            }
        }

        let url = url.ok_or(Error::ParseArgs("No url given".to_string()))?;

        Ok(Self { url })
    }
}

pub fn import(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;
    let song = ultimate_guitar_to_song(&config.url)?;
    for line in song.to_string() {
        println!("{}", line);
    }
    Ok(())
}
