use std::env;
use std::path::PathBuf;

use super::super::Error;

pub struct Config {
    pub root_path: Option<PathBuf>,
    pub setlist_path: Option<PathBuf>,
    pub url: Option<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut root_path_or_url: Option<String> = None;
        let mut setlist_path: Option<PathBuf> = None;
        let mut root_path: Option<PathBuf> = None;
        let mut url: Option<String> = None;
        let mut remote = false;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-l" => match args.next() {
                    Some(path) => setlist_path = Some(PathBuf::from(path)),
                    None => {
                        return Err(Error::ParseArgs("No value for option -l given".to_string()))
                    }
                },
                "--remote" => remote = true,
                f => root_path_or_url = Some(f.to_string()),
            }
        }

        if remote {
            url = Some(root_path_or_url.unwrap_or("http://127.0.0.1:8000".to_string()));
        } else {
            root_path = Some(PathBuf::from(root_path_or_url.unwrap_or(".".to_string())));
            setlist_path = Some(
                setlist_path.unwrap_or(
                    root_path
                        .clone()
                        .expect("root_path ist set")
                        .join("setlists"),
                ),
            );
        }
        Ok(Self {
            url,
            root_path,
            setlist_path,
        })
    }
}
