use super::{
    Chord, ChordIterator, Directive, Line, LineIterator, NotTransposable, Text, TextChordTrans,
    TranslationChord, TranslationText, Transposable,
};

const SCALE: [[&'static str; 12]; 2] = [
    [
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
    ],
    [
        "A", "Bb", "Cb", "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab",
    ],
];

fn chord_to_level(chord: &str) -> i8 {
    let mut chars = chord.chars();
    let mut level: i8 = match chars.next() {
        Some('A') => 0,
        Some('B') => 2,
        Some('C') => 3,
        Some('D') => 5,
        Some('E') => 7,
        Some('F') => 8,
        Some('G') => 10,
        _ => 0,
    };
    level += match chars.next() {
        Some('b') => -1,
        Some('#') => 1,
        _ => 0,
    };
    level.rem_euclid(12)
}

fn level_to_scale(level: i8) -> usize {
    match level.rem_euclid(12) {
        0 | 2 | 3 | 5 | 7 | 10 => 0,
        _ => 1,
    }
}

fn transpose_chord_part(chord: &str, halftones: i8, scale: usize) -> &str {
    SCALE[scale][(chord_to_level(chord) + halftones).rem_euclid(12) as usize]
}

fn transpose_chord(chord: &str, halftones: i8, scale: usize) -> String {
    ChordIterator::new(chord)
        .map(|item| match item {
            Transposable(item) => transpose_chord_part(item, halftones, scale),
            NotTransposable(item) => item,
        })
        .collect()
}

fn transpose_line(line: &Line, halftones: i8, scale: usize) -> Line {
    if let TextChordTrans(line) = line {
        let mut line_new = String::new();
        let mut added_amp = false;
        for part in LineIterator::new(&line) {
            match part {
                Chord(chord) => {
                    line_new.push_str("[");
                    line_new.push_str(&transpose_chord(chord, halftones, scale));
                    line_new.push_str("]");
                }
                Text(text) => line_new.push_str(text),
                TranslationText(translation_text) => {
                    if !added_amp {
                        line_new.push_str(" & ");
                        added_amp = true;
                    }
                    line_new.push_str(translation_text);
                }
                TranslationChord(chord) => {
                    if !added_amp {
                        line_new.push_str(" & ");
                        added_amp = true;
                    }
                    line_new.push_str("[");
                    line_new.push_str(&transpose_chord(chord, halftones, scale));
                    line_new.push_str("]");
                }
            }
        }
        TextChordTrans(line_new)
    } else {
        line.clone()
    }
}

pub struct Transpose<I>
where
    I: Iterator<Item = Line>,
{
    iter: I,
    key_new: Option<i8>,
    scale: Option<usize>,
    halftones: Option<i8>,
}

impl<I> Transpose<I>
where
    I: Iterator<Item = Line>,
{
    fn new(iter: I, key: &str) -> Self {
        // Self with offset
        if let Some(idx) = key.find(':') {
            if &key[..idx] == "Self" {
                if let Ok(offset) = key[idx + 1..].to_string().parse::<i8>() {
                    return Self {
                        iter,
                        key_new: None,
                        scale: None,
                        halftones: Some(offset),
                    };
                }
            }
        }

        // Self
        if key == "Self" {
            return Self {
                iter,
                key_new: None,
                scale: None,
                halftones: Some(0),
            };
        }

        // other
        let level = chord_to_level(key);
        Self {
            iter,
            key_new: Some(level),
            scale: Some(level_to_scale(level)),
            halftones: None,
        }
    }
}

impl<I> Iterator for Transpose<I>
where
    I: Iterator<Item = Line>,
{
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.iter.next();
        match &line {
            Some(Directive((key, value))) => {
                if key == "key" {
                    if self.scale == None {
                        let offset = self.halftones.unwrap_or(0);
                        self.scale = Some(
                            level_to_scale((chord_to_level(value)) as i8 + offset).rem_euclid(12)
                                as usize,
                        );
                    }
                    if let Some(key_new) = self.key_new {
                        self.halftones = Some((key_new - chord_to_level(value)).rem_euclid(12));
                    }
                    if let Some(halftones) = self.halftones {
                        if let Some(scale) = self.scale {
                            return Some(Directive((
                                key.to_string(),
                                transpose_chord_part(value, halftones, scale).to_string(),
                            )));
                        }
                    }
                }
            }
            Some(TextChordTrans(_)) => {
                if let Some(halftones) = self.halftones {
                    if let Some(scale) = self.scale {
                        if let Some(line) = line {
                            return Some(transpose_line(&line, halftones, scale));
                        }
                    }
                }
            }
            _ => (),
        }
        line
    }
}

pub trait IntoTranspose: Iterator {
    fn transpose(self, key: &str) -> Transpose<Self>
    where
        Self: Sized + Iterator<Item = Line>,
    {
        Transpose::new(self.into_iter(), key)
    }
}
impl<I> IntoTranspose for I where I: Sized + Iterator<Item = Line> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_chord_after_and_translation() {
        let line = Line::TextChordTrans("[G]Hello[C] & [G]Hallo[C]".to_string());
        assert_eq!(transpose_line(&line, 0, 0), line);
    }

    #[test]
    fn fn_directive() {
        let line = Line::Directive(("key".to_string(), "value".to_string()));
        assert_eq!(transpose_line(&line, 0, 0), line);
    }

    #[test]
    fn fn_text_chord_trans_self() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(&line, 0, 0), line);
    }

    #[test]
    fn fn_text_chord_trans_plus_one() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let line_new = Line::TextChordTrans("This is a [Db]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(&line, 1, 1), line_new);
    }

    #[test]
    fn fn_text_chord_trans_minus() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(&line, -120, 0), line);
    }

    #[test]
    fn empty() {
        let vec = std::iter::empty::<Line>()
            .transpose("Self")
            .collect::<Vec<Line>>();
        assert_eq!(vec, vec!());
    }

    #[test]
    fn with_key() {
        let directive = Line::Directive(("key".to_string(), "C".to_string()));
        let directive2 = Line::Directive(("key".to_string(), "Db".to_string()));
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let text_chord_trans_new =
            Line::TextChordTrans("This is a [Db]line & Das ist eine Zeile".to_string());
        let vec = vec![directive.clone(), text_chord_trans]
            .into_iter()
            .transpose("Db")
            .collect::<Vec<Line>>();
        let vec_new = vec![directive2, text_chord_trans_new];
        assert_eq!(vec, vec_new);
    }

    #[test]
    fn without_key() {
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let vec = vec![text_chord_trans.clone()]
            .into_iter()
            .transpose("Db")
            .collect::<Vec<Line>>();
        let vec_new = vec![text_chord_trans];
        assert_eq!(vec, vec_new);
    }

    #[test]
    fn offset() {
        let directive = Line::Directive(("key".to_string(), "C".to_string()));
        let directive2 = Line::Directive(("key".to_string(), "Db".to_string()));
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let text_chord_trans_new =
            Line::TextChordTrans("This is a [Db]line & Das ist eine Zeile".to_string());
        let vec = vec![directive.clone(), text_chord_trans]
            .into_iter()
            .transpose("Self:1")
            .collect::<Vec<Line>>();
        let vec_new = vec![directive2, text_chord_trans_new];
        assert_eq!(vec, vec_new);
    }

    #[test]
    fn offset_negative() {
        let directive = Line::Directive(("key".to_string(), "C".to_string()));
        let directive2 = Line::Directive(("key".to_string(), "Bb".to_string()));
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let text_chord_trans_new =
            Line::TextChordTrans("This is a [Bb]line & Das ist eine Zeile".to_string());
        let vec = vec![directive.clone(), text_chord_trans]
            .into_iter()
            .transpose("Self:-2")
            .collect::<Vec<Line>>();
        let vec_new = vec![directive2, text_chord_trans_new];
        assert_eq!(vec, vec_new);
    }

    #[test]
    fn offset_bullshit() {
        let directive = Line::Directive(("key".to_string(), "C".to_string()));
        let directive2 = Line::Directive(("key".to_string(), "A".to_string()));
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let text_chord_trans_new =
            Line::TextChordTrans("This is a [A]line & Das ist eine Zeile".to_string());
        let vec = vec![directive.clone(), text_chord_trans]
            .into_iter()
            .transpose("Self:bullshit")
            .collect::<Vec<Line>>();
        let vec_new = vec![directive2, text_chord_trans_new];
        assert_eq!(vec, vec_new);
    }
}
