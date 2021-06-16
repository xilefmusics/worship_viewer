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

fn find_closing(s: &str) -> Option<usize> {
    let mut cnt: usize = 0;
    let mut index: usize = 0;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                cnt += 1;
                index += 1
            }
            ')' | ']' | '}' => {
                cnt -= 1;
                if cnt == 0 {
                    return Some(index);
                }
                index += 1;
            }
            c => index += c.len_utf8(),
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub song_name: String,
    pub artist_name: String,
    pub url: String,
}

pub fn search(search_str: &str) -> Result<Vec<SearchResult>, Error> {
    // format url
    let url = format!(
        "https://www.ultimate-guitar.com/search.php?search_type=title&value={}",
        search_str.replace(" ", "%20")
    );

    // get html
    let res = reqwest::blocking::get(url)?;
    let text = res.text()?;
    let html = html_escape::decode_html_entities(&text);

    // find content
    let mut content = &html[html
        .find("js-store")
        .ok_or(Error::SongParse("No js-store in html".to_string()))?..];
    content = &content[content
        .find("\"data\"")
        .ok_or(Error::SongParse("No data in html".to_string()))?..];
    content = &content[content
        .find("\"results\"")
        .ok_or(Error::SongParse("No results in html".to_string()))?..];
    content = &content[content
        .find("[")
        .ok_or(Error::SongParse("No [ in html".to_string()))?..];
    content = &content[..find_closing(&content)
        .ok_or(Error::SongParse("No matching ] in html".to_string()))?
        + 1];

    // parse search results
    let mut results: Vec<SearchResult> = Vec::new();
    loop {
        content = match content.find("{") {
            Some(idx) => &content[idx..],
            None => break,
        };
        let closing_idx =
            find_closing(&content).ok_or(Error::SongParse("No matching } in html".to_string()))?;
        let song = &content[1..closing_idx];
        content = &content[closing_idx..];

        let song_name = &song[song.find("\"song_name\"").ok_or(Error::SongParse(
            "No song_name in search_result".to_string(),
        ))? + 13..];
        let song_name = song_name[..song_name.find("\"").ok_or(Error::SongParse(
            "No closing \" for song_name in search_result".to_string(),
        ))?]
            .to_string();

        let artist_name = &song[song.find("\"artist_name\"").ok_or(Error::SongParse(
            "No artist_name in search_result".to_string(),
        ))? + 15..];
        let artist_name = artist_name[..artist_name.find("\"").ok_or(Error::SongParse(
            "No closing \" for artist_name in search_result".to_string(),
        ))?]
            .to_string();

        let url = &song[song.find("\"tab_url\"").ok_or(Error::SongParse(
            "No artist_url in search_result".to_string(),
        ))? + 11..];
        let url = url[..url.find("\"").ok_or(Error::SongParse(
            "No closing \" for artist_url in search_result".to_string(),
        ))?]
            .to_string();

        results.push(SearchResult {
            song_name,
            artist_name,
            url,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_closing() {
        assert_eq!(find_closing("()"), Some(1));
        assert_eq!(find_closing("(()(()))(())"), Some(7));
        assert_eq!(find_closing("(()(())"), None);
        assert_eq!(find_closing("(ä)"), Some(3));
    }

    //#[test]
    fn _test_search() {
        let results = search("I need a ghost");
        println!("{:?}", results);
        assert!(false);
    }

    //#[test]
    fn _test_url_to_song() {
        let url = "https://tabs.ultimate-guitar.com/tab/2427469";
        let song = url_to_song(url);
        println!("{:?}", song);
        assert!(false);
    }
}
