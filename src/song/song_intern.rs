use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use super::line::{
    IterExtToMulti, IterExtToSection, IterExtToString, IterExtToWp, IterExtTranspose, Section,
    WpLine,
};

use super::Error;
use super::Song;

#[derive(Debug, Clone)]
pub struct SongIntern {
    pub title: String,
    pub artist: String,
    pub key: String,
    pub lines: Vec<WpLine>,
    pub path: Option<PathBuf>,
}

impl SongIntern {
    pub fn load(path: PathBuf) -> Result<Self, Error> {
        let mut title: Option<String> = None;
        let mut artist: Option<String> = None;
        let mut key: Option<String> = None;

        let lines = fs::read_to_string(&path)?
            .lines()
            .to_wp()
            .map(|line| {
                if let WpLine::Directive((k, v)) = &line {
                    match k.as_str() {
                        "title" => title = Some(v.clone()),
                        "artist" => artist = Some(v.clone()),
                        "key" => key = Some(v.clone()),
                        _ => (),
                    }
                }
                line
            })
            .collect::<Vec<WpLine>>();

        let title = title.ok_or(Error::SongParse("No title given".to_string()))?;
        let artist = artist.ok_or(Error::SongParse("No artist given".to_string()))?;
        let key = key.ok_or(Error::SongParse("No key given".to_string()))?;
        let path = Some(path);
        Ok(Self {
            title,
            artist,
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
        let artist = self.artist.clone();
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
            artist,
            key,
            sections,
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        let mut file = File::create(self.path.clone().ok_or(Error::NoPath)?)?;
        for line in self.lines.clone().into_iter().to_string() {
            file.write_fmt(format_args!("{}\n", line))?;
        }
        Ok(())
    }

    pub fn transpose(&self, key: String) -> Self {
        let lines = self
            .lines
            .clone()
            .into_iter()
            .transpose(&key)
            .collect::<Vec<WpLine>>();
        let title = self.title.clone();
        let artist = self.artist.clone();
        let path = self.path.clone();
        Self {
            title,
            artist,
            key,
            lines,
            path,
        }
    }
}
