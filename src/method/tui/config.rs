use std::env;

use super::super::Error;

pub struct Config {
    pub folder: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let folder = match args.next() {
            Some(folder) => folder,
            None => String::from("."),
        };
        Ok(Self { folder })
    }
}
