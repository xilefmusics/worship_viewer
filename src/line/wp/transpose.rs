use super::{
    Chord, ChordIterator, Directive, Line, LineIterator, NotTransposable, Text, TextChordTrans,
    Translation, Transposable,
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
        0 | 2 | 5 | 7 | 10 => 0,
        _ => 1,
    }
}

fn transpose_chord_part(chord: &str, halftones: i8, scale: usize) -> &str {
    SCALE[scale][(chord_to_level(chord) + halftones).rem_euclid(12) as usize]
}

fn transpose_chord(chord: &str, halftones: i8, scale: usize) -> String {
    let mut chord_new = String::new();
    for part in ChordIterator::new(chord) {
        match part {
            Transposable(t) => chord_new.push_str(transpose_chord_part(t, halftones, scale)),
            NotTransposable(t) => chord_new.push_str(t),
        }
    }
    chord_new
}

fn transpose_line(line: Line, halftones: i8, scale: usize) -> Line {
    if let TextChordTrans(line) = line {
        let mut line_new = String::new();
        for part in LineIterator::new(&line) {
            match part {
                Chord(chord) => {
                    line_new.push_str("[");
                    line_new.push_str(&transpose_chord(chord, halftones, scale));
                    line_new.push_str("]");
                }
                Text(text) => line_new.push_str(text),
                Translation(translation) => {
                    line_new.push_str(" & ");
                    line_new.push_str(translation);
                }
            }
        }
        TextChordTrans(line_new)
    } else {
        line
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
        if key == "Self" {
            Self {
                iter,
                key_new: None,
                scale: None,
                halftones: Some(0),
            }
        } else {
            let level = chord_to_level(key);
            Self {
                iter,
                key_new: Some(level),
                scale: Some(level_to_scale(level)),
                halftones: None,
            }
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
                    if let None = self.scale {
                        self.scale = Some(level_to_scale(chord_to_level(value)));
                    }
                    if let Some(key_new) = self.key_new {
                        self.halftones = Some((key_new - chord_to_level(value)).rem_euclid(12));
                    }
                }
            }
            Some(TextChordTrans(_)) => {
                if let Some(halftones) = self.halftones {
                    if let Some(scale) = self.scale {
                        if let Some(line) = line {
                            return Some(transpose_line(line, halftones, scale));
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
    fn fn_directive() {
        let line = Line::Directive(("key".to_string(), "value".to_string()));
        assert_eq!(transpose_line(line.clone(), 0, 0), line);
    }

    #[test]
    fn fn_text_chord_trans_self() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(line.clone(), 0, 0), line);
    }

    #[test]
    fn fn_text_chord_trans_plus_one() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let line_new = Line::TextChordTrans("This is a [Db]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(line, 1, 1), line_new);
    }

    #[test]
    fn fn_text_chord_trans_minus() {
        let line = Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        assert_eq!(transpose_line(line.clone(), -120, 0), line);
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
        let text_chord_trans =
            Line::TextChordTrans("This is a [C]line & Das ist eine Zeile".to_string());
        let text_chord_trans_new =
            Line::TextChordTrans("This is a [Db]line & Das ist eine Zeile".to_string());
        let vec = vec![directive.clone(), text_chord_trans]
            .into_iter()
            .transpose("Db")
            .collect::<Vec<Line>>();
        let vec_new = vec![directive, text_chord_trans_new];
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
}
