use super::Key;
use super::Line;

struct Meta {
    title: String,
    artist: String,
    key: Key,
}

impl Meta {
    pub fn from_string(string: &str) -> Result<Meta, String> {
        let mut title: Option<String> = None;
        let mut artist: Option<String> = None;
        let mut key: Option<Key> = None;
        for line in string.lines() {
            if let Some(idx1) = line.find("{") {
                if let Some(idx2) = line.find(":") {
                    if let Some(idx3) = line.find("}") {
                        match line[idx1+1..idx2].trim() {
                            "title" => title = Some(String::from(line[idx2+1..idx3].trim())),
                            "artist" => artist = Some(String::from(line[idx2+1..idx3].trim())),
                            "key" => key = Some(Key::from_string(line[idx2+1..idx3].trim())?),
                            _ => (),
                        }
                    }
                }
            }
        }
        let title = title.ok_or(String::from("no title given"))?;
        let artist = artist.ok_or(String::from("no artist given"))?;
        let key = key.ok_or(String::from("no key given"))?;
        Ok(Meta{title, artist, key})
    }
}

struct Section {
    keyword: Option<String>,
    lines: Vec<Line>,
}

impl Section {
    pub fn new(keyword: Option<String>, lines: Vec<&str>, key: &Key) -> Result<Section, String> {
        let lines: Result<Vec<Line>, String> = lines.iter().map(|line| Line::from_string(line, &key)).collect();
        let lines = lines?;
        Ok(Section{keyword, lines})
    }
}

pub struct Song {
    meta: Meta,
    sections: Vec<Section>,
}

impl Song {
    pub fn from_string(string: &str) -> Result<Song, String> {
        let meta = Meta::from_string(string)?;
        let mut sections : Vec<Section> = vec!();
        let mut keyword: Option<String> = None;
        for (line_idx, line) in string.lines().enumerate() {
            if let Some(idx1) = line.find("{") {
                if let Some(idx2) = line.find(":") {
                    if let Some(idx3) = line.find("}") {
                        let arg = line[idx2+1..idx3].trim();
                        if let Some (idx4) = arg.find(" ") {
                            if (&arg[..idx4] == "section") {
                                keyword = Some(String::from(&arg[idx4+1..]));
                            }
                        }
                    }
                }
            } else {
                let mut lines: Vec<&str> = vec!();
                for line in string.lines().skip(line_idx) {
                    if let Some(idx1) = line.find("{") {
                        break;
                    }
                    lines.push(line);
                }
                sections.push(Section::new(keyword.clone(), lines, &meta.key)?);
            }
        }
        Ok(Song{meta, sections})
    }
}
