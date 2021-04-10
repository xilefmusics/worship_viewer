use super::{Key, Chord, Textline};


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
            let v1 = &(first.clone())[..chord_start];
            let v2 = &(first.clone())[chord_stop+1..];
            first = String::from(v1.clone());
            first.push_str(v2);
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

    pub fn textlines(&self, key: &Key) -> Result<Vec<Textline>, String> {
        let mut textlines: Vec<Textline> = Vec::new();
        if self.chords.len() > 0 {
            let mut chordstr: String = String::new();
            for (chord_offset, chord) in self.chord_offsets.iter().zip(self.chords.iter()) {
                let len = chordstr.len();
                for i in 0..*chord_offset-len {
                    chordstr.push_str(" ");
                }
                chordstr.push_str(&chord.to_string(key)?);
            }
            textlines.push(Textline::CHORD(chordstr));
        }
        if let Some(text) = self.text.clone() {
            textlines.push(Textline::TEXT(text));
        }
        if let Some(translation) = self.translation.clone() {
            textlines.push(Textline::TRANSLATION(translation));
        }
        Ok(textlines)
    }
}
