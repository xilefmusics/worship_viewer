use super::Error;
use std::env;

use crate::song::import::ultimate_guitar_search;

pub struct Config {
    pub search_str: String,
}
impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut search_str: Option<String> = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                ss => search_str = Some(ss.to_string()),
            }
        }

        let search_str =
            search_str.ok_or(Error::ParseArgs("No search string given".to_string()))?;

        Ok(Self { search_str })
    }
}

pub fn online_search(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;
    for search_result in ultimate_guitar_search(&config.search_str)? {
        println!(
            "{}\t{}\t{}",
            search_result.song_name, search_result.artist_name, search_result.url
        )
    }
    Ok(())
}
