use std::fs;
use std::path::PathBuf;

use crate::line::{
    IterExtToMulti, IterExtToSection, IterExtToWp, IterExtTranspose, Section, WpLine,
};

use super::{Error, SectionSong};

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub key: String,
    pub lines: Vec<WpLine>,
    pub path: PathBuf,
}

impl Song {
    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let mut title: Option<String> = None;
        let mut key: Option<String> = None;

        let lines = fs::read_to_string(&path)
            .map_err(|_| Error::IO)?
            .lines()
            .to_wp()
            .map(|line| {
                if let WpLine::Directive((k, v)) = &line {
                    match k.as_str() {
                        "title" => title = Some(v.clone()),
                        "key" => key = Some(v.clone()),
                        _ => (),
                    }
                }
                line
            })
            .collect::<Vec<WpLine>>();

        let title = title.ok_or(Error::SongParse("No title given".to_string()))?;
        let key = key.ok_or(Error::SongParse("No key given".to_string()))?;
        Ok(Self {
            title,
            key,
            lines,
            path,
        })
    }

    pub fn load_all(path: &PathBuf) -> Result<Vec<Self>, Error> {
        let mut songs = fs::read_dir(path)
            .map_err(|_| Error::IO)?
            .map(|res| res.map(|e| e.path()))
            .filter(|path| path.is_ok() && !path.as_ref().unwrap().is_dir())
            .map(|path| Self::load(path.map_err(|_| Error::IO)?.clone()))
            .collect::<Result<Vec<Self>, Error>>()?;
        songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(songs)
    }

    pub fn lines_ref(&self) -> &Vec<WpLine> {
        &self.lines
    }

    pub fn lines(&self) -> Vec<WpLine> {
        self.lines.clone()
    }

    pub fn transpose(&self, key: String) -> Self {
        let title = self.title.clone();
        let lines = self
            .lines()
            .into_iter()
            .transpose(&key)
            .map(|line| {
                if let WpLine::Directive((k, _)) = &line {
                    match k.as_str() {
                        "key" => WpLine::Directive(("key".to_string(), key.clone())),
                        _ => line,
                    }
                } else {
                    line
                }
            })
            .collect::<Vec<WpLine>>();
        let path = self.path.clone();
        Self {
            title,
            lines,
            path,
            key,
        }
    }

    pub fn to_section_song(&self, key: &str) -> Result<SectionSong, Error> {
        let title = self.title.clone();
        let sections = self
            .lines()
            .into_iter()
            .transpose(key)
            .to_multi()
            .to_section()
            .collect::<Vec<Section>>();
        Ok(SectionSong::new(title, sections))
    }
}
