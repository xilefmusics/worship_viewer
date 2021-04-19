use std::fs;
use std::path::PathBuf;

use super::super::super::line::{IterExtToWp, WpLine};

use super::super::Error;

use super::Config;

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
}

impl Song {
    pub fn load_vec(config: &Config) -> Result<Vec<Self>, Error> {
        let mut songs = fs::read_dir(&config.folder)
            .map_err(|_| Error::IO)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| {
                if let Ok(path) = path {
                    !path.is_dir()
                } else {
                    false
                }
            })
            .map(|path| {
                let path = path.map_err(|_| Error::IO)?.clone();
                let line = fs::read_to_string(&path)
                    .map_err(|_| Error::IO)?
                    .lines()
                    .to_wp()
                    .find(|line| match line {
                        WpLine::Directive((key, _)) => match key.as_str() {
                            "title" => true,
                            _ => false,
                        },
                        _ => false,
                    });

                let title = match line {
                    Some(WpLine::Directive((_, title))) => title,
                    _ => String::new(),
                };
                Ok(Song { title, path })
            })
            .collect::<Result<Vec<Song>, Error>>()?;

        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(songs)
    }
}
