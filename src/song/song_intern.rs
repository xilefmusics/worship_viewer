use std::fs;
use std::path::PathBuf;

use super::line::{
    IterExtToMulti, IterExtToSection, IterExtToWp, IterExtTranspose, Section, WpLine,
};

use super::Error;
use super::Song;

#[derive(Debug, Clone)]
pub struct SongIntern {
    pub title: String,
    pub key: String,
    pub lines: Vec<WpLine>,
    pub path: PathBuf,
}

impl SongIntern {
    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let mut title: Option<String> = None;
        let mut key: Option<String> = None;

        let lines = fs::read_to_string(&path)?
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

    pub fn lines(&self) -> Vec<WpLine> {
        self.lines.clone()
    }

    pub fn to_section_song(&self, key: &str) -> Song {
        let title = self.title.clone();
        let key = key.to_string();
        let sections = self
            .lines()
            .into_iter()
            .transpose(&key)
            .to_multi()
            .to_section()
            .collect::<Vec<Section>>();
        Song {
            title,
            key,
            sections,
        }
    }
}
