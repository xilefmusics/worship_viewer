use std::env;
use std::path::PathBuf;

use super::super::Error;

pub struct Config {
    pub root_path: PathBuf,
    pub setlist_path: Option<PathBuf>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut root_path: Option<String> = None;
        let mut setlist_path: Option<PathBuf> = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-l" => match args.next() {
                    Some(path) => setlist_path = Some(PathBuf::from(path)),
                    None => {
                        return Err(Error::ParseArgs("No value for option -l given".to_string()))
                    }
                },
                f => root_path = Some(f.to_string()),
            }
        }
        let root_path = PathBuf::from(root_path.unwrap_or(".".to_string()));
        Ok(Self {
            root_path,
            setlist_path,
        })
    }
}
