use std::fs;
use std::path::PathBuf;

use super::super::super::line::{IterExtToWp, WpLine};

use super::super::Error;

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub key: String,
    pub path: PathBuf,
}

impl Song {
    pub fn load_all(path: &PathBuf) -> Result<Vec<Self>, Error> {
        let mut songs = fs::read_dir(path)
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
                let key = "Self".to_string();
                Ok(Self { title, key, path })
            })
            .collect::<Result<Vec<Self>, Error>>()?;

        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(songs)
    }
}
