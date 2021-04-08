use super::Chord;
use super::Key;

pub struct Line {
    text: Option<String>,
    translation: Option<String>,
    chords: Vec<Chord>,
    chord_offsets: Vec<usize>,
}

impl Line {
    pub fn from_string(string: &str, key: &Key) -> Result<Line, String> {
        let mut chord_offsets: Vec<usize> = vec!();
        let mut chords: Vec<Chord> = vec!();
        let mut iter = string.split("&");
        let mut first = iter.next().unwrap_or("").trim().to_string();
        let translation = match iter.next() {
            Some(translation) => Some(String::from(translation.trim())),
            None => None,
        };
        while let Some(chord_start) = first.find("[") {
            let chord_stop = match first.find("]") {
                Some(chord_stop) => chord_stop,
                None => return Err(format!("\"{}\" is not a valid line", string)),
            };
            chord_offsets.push(chord_start);
            chords.push(Chord::from_string(&first[chord_start+1..chord_stop], key));
            first = first.chars().take(chord_start).chain(first.chars().skip(chord_stop+1)).collect();
        }
        let text: Option<String> = Some(first.to_string());
        Ok(Line{text, translation, chords, chord_offsets})
    }

    pub fn to_string(&self, key: &Key) -> Result<String, String> {
        let mut result = match &self.text {
            Some(text) => {
                let mut result: String = text.chars().collect();
                for (chord, offset) in self.chords.iter().zip(self.chord_offsets.iter()) {
                    result = result.chars().take(*offset).chain("[".chars()).chain(chord.to_string(&key)?.chars()).chain("]".chars()).chain(result.chars().skip(*offset)).collect();
                }
                result
            },
            None => {
                match self.chords.iter().next() {
                    Some(chord) => chord.to_string(&key)?,
                    None => return Err(String::from("line is empty")),
                }
            },
        };
        if let Some(translation) = &self.translation {
            result.push_str(" & ");
            result.push_str(&translation);
        }
        Ok(result)
    }
}
