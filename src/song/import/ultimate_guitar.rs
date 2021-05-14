use crate::song::line::Multiline as Line;
use crate::song::line::{IterExtToSection, IterExtUnflatten};
use crate::song::{Error, Section, Song};

fn classify_line(mut line: String) -> Result<Option<Line>, Error> {
    if line.find("[ch]").is_some() {
        line = line.replace("[ch]", "");
        line = line.replace("[/ch]", "");
        line = line.replace("[tab]", "");
        line = line.replace("[/tab]", "");
        Ok(Some(Line::Chord(line)))
    } else if let Some('[') = line.chars().next() {
        line = line[1..line.find("]").ok_or(Error::SongParse("No ]".to_string()))?].to_string();
        Ok(Some(Line::Keyword(line)))
    } else {
        line = line.replace("[ch]", "");
        line = line.replace("[/ch]", "");
        line = line.replace("[tab]", "");
        line = line.replace("[/tab]", "");
        if line.len() > 0 {
            Ok(Some(Line::Text(line)))
        } else {
            Ok(None)
        }
    }
}

pub fn url_to_song(url: &str) -> Result<Song, Error> {
    // get html
    let res = reqwest::blocking::get(url)?;
    let text = res.text()?;
    let html = html_escape::decode_html_entities(&text);

    // find start of content
    let mut content = &html[html
        .find("js-store")
        .ok_or(Error::SongParse("No js-store in html".to_string()))?..];
    content = &content[content
        .find("wiki_tab")
        .ok_or(Error::SongParse("No wiki_tab in html".to_string()))?..];
    content = &content[content
        .find("content")
        .ok_or(Error::SongParse("No content in html".to_string()))?..];
    content = &content[content
        .find(":")
        .ok_or(Error::SongParse("No :".to_string()))?..];
    content = &content[content
        .find("\"")
        .ok_or(Error::SongParse("No opening \"".to_string()))?
        + 1..];
    content = &content[..content
        .find("\"")
        .ok_or(Error::SongParse("No closing\"".to_string()))?];

    // parse content into vector of Lines
    let mut lines: Vec<Line> = Vec::new();
    while let Some(idx) = content.find("\\r\\n") {
        if let Some(line) = classify_line(content[..idx].to_string())? {
            lines.push(line);
        }
        content = &content[idx + 4..];
    }
    if let Some(line) = classify_line(content.to_string())? {
        lines.push(line);
    }

    // construct sections
    let sections = lines
        .into_iter()
        .unflatten()
        .to_section()
        .collect::<Vec<Section>>();

    // find song_name
    let mut song_name = &html[html
        .find("song_name")
        .ok_or(Error::SongParse("No song_name in html".to_string()))?..];
    song_name = &song_name[song_name
        .find(":")
        .ok_or(Error::SongParse("No :".to_string()))?..];
    song_name = &song_name[song_name
        .find("\"")
        .ok_or(Error::SongParse("No opening \"".to_string()))?
        + 1..];
    song_name = &song_name[..song_name
        .find("\"")
        .ok_or(Error::SongParse("No closing \"".to_string()))?];

    //find artist_name
    let mut artist_name = &html[html
        .find("artist_name")
        .ok_or(Error::SongParse("No artist_name in html".to_string()))?..];
    artist_name = &artist_name[artist_name
        .find(":")
        .ok_or(Error::SongParse("No :".to_string()))?..];
    artist_name = &artist_name[artist_name
        .find("\"")
        .ok_or(Error::SongParse("No opening\"".to_string()))?
        + 1..];
    artist_name = &artist_name[..artist_name
        .find("\"")
        .ok_or(Error::SongParse("No closing \"".to_string()))?];

    // find tonality
    let mut tonality = &html[html
        .find("tonality")
        .ok_or(Error::SongParse("No tonality in html".to_string()))?..];
    tonality = &tonality[tonality
        .find(":")
        .ok_or(Error::SongParse("No :".to_string()))?..];
    tonality = &tonality[tonality
        .find("\"")
        .ok_or(Error::SongParse("No opening \"".to_string()))?
        + 1..];
    tonality = &tonality[..tonality
        .find("\"")
        .ok_or(Error::SongParse("No closing \"".to_string()))?];

    // return song
    Ok(Song {
        title: song_name.to_string(),
        artist: artist_name.to_string(),
        key: tonality.to_string(),
        sections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    fn _test_url_to_song() {
        let url = "https://tabs.ultimate-guitar.com/tab/2427469";
        let song = url_to_song(url);
        println!("{:?}", song);
        assert!(false);
    }
}
