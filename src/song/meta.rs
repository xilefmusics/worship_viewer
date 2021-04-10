use super::Key;

pub struct Meta {
    pub title: String,
    pub artist: String,
    pub key: Key,
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
