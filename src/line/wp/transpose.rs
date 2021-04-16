use super::{
    Chord, ChordIterator, Directive, Line, LineIterator, NotTransposable, Text, TextChordTrans,
    Translation, Transposabel,
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

pub struct Transpose<I>
where
    I: Iterator<Item = Line>,
{
    iter: I,
    key_new: Option<i8>,
    key_old: Option<i8>,
}

impl<I> Transpose<I>
where
    I: Iterator<Item = Line>,
{
    fn new(iter: I, key: &str) -> Self {
        let key_new = match key {
            "Self" => None,
            key => Some(chord_to_level(key)),
        };
        Self {
            iter,
            key_new,
            key_old: None,
        }
    }

    fn t(&self, line: Option<Line>) -> Option<Line> {
        let line = line?;
        let key_new = match self.key_new {
            Some(key) => key,
            None => return Some(line),
        };
        let key_old = self.key_old?;
        if let TextChordTrans(line) = line {
            let scale = match key_new {
                0 | 2 | 3 | 5 | 7 | 10 => 0,
                _ => 1,
            };
            let mut line_new = String::new();
            for item in LineIterator::new(&line) {
                match item {
                    Chord(c) => {
                        line_new.push_str("[");
                        for item in ChordIterator::new(&c) {
                            match item {
                                Transposabel(s) => line_new.push_str(
                                    SCALE[scale][(chord_to_level(&s) + key_new - key_old)
                                        .rem_euclid(12)
                                        as usize],
                                ),
                                NotTransposable(s) => line_new.push_str(&s),
                            }
                        }
                        line_new.push_str("]");
                    }
                    Text(t) => line_new.push_str(&t),
                    Translation(t) => {
                        line_new.push_str(" & ");
                        line_new.push_str(&t);
                    }
                }
            }
            return Some(TextChordTrans(line_new));
        }
        Some(line)
    }
}

impl<I> Iterator for Transpose<I>
where
    I: Iterator<Item = Line>,
{
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        match &item {
            Some(Directive((key, value))) => {
                if key == "key" {
                    self.key_old = Some(chord_to_level(&value))
                }
                item
            }
            Some(TextChordTrans(_)) => self.t(item),
            _ => item,
        }
    }
}

pub trait IntoTranspose: Iterator {
    fn transpose(self, key: &str) -> Transpose<Self>
    where
        Self: Sized + Iterator<Item = Line>,
    {
        Transpose::new(self, key)
    }
}
impl<I> IntoTranspose for I where I: Sized + Iterator<Item = Line> {}
