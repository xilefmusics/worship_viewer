#[derive(Debug)]
enum WorshipProLine {
    Directive((String, String)),
    TextChordTrans(String),
    Empty,
}

#[derive(Debug)]
enum Multiline {
    Keyword(String),
    Chord(String),
    Text(String),
    Translation(String),
}

#[derive(Debug)]
enum ChordIteratorItem {
    Transposabel(String),
    NotTransposable(String),
}

#[derive(Debug)]
struct ChordIterator {
    chord: String,
    index: usize,
}
impl ChordIterator {
    fn new(chord: &str) -> Self {
        Self{chord: chord.to_string(), index: 0}
    }
}
impl Iterator for ChordIterator {
    type Item = ChordIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let string: String = self.chord[self.index..].to_string();
        let mut iter = string.chars();
        let first = iter.next()?;
        let transposable = match first {
            'A'|'B'|'C'|'D'|'E'|'F'|'G' => true,
            _ => false,
        };


        if transposable {
            match iter.next() {
                Some('b') => {
                    self.index += 2;
                    Some(ChordIteratorItem::Transposabel([first, 'b'].iter().collect()))
                },
                Some('#') => {
                    self.index += 2;
                    Some(ChordIteratorItem::Transposabel([first, '#'].iter().collect()))
                },
                _ => {
                    self.index += 1;
                    Some(ChordIteratorItem::Transposabel(first.to_string()))
                },
            }
        } else {
            let mut result = String::new();
            for c in string.chars() {
                match c {
                    'A'|'B'|'C'|'D'|'E'|'F'|'G' => break,
                    c => result.push_str(&c.to_string()),
                }
            }
            self.index += result.len();
            Some(ChordIteratorItem::NotTransposable(result))
        }
    }
}

#[derive(Debug)]
enum LineIteratorItem {
    Chord(String),
    Text(String),
    Translation(String),
}

#[derive(Debug)]
struct LineIterator {
    line: String,
    index: usize,
}
impl LineIterator {
    fn new(line: &str) -> Self {
        LineIterator{line: line.to_string(), index: 0}
    }
}
impl Iterator for LineIterator {
    type Item = LineIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.line.len() - self.index;
        let line = &self.line[self.index..];

        // End
        if len <= 0 {
            return None;
        }

        // Chord
        if line.starts_with('[') {
            if let Some(end_idx) = line.find("]") {
                self.index += end_idx + 1;
                return Some(LineIteratorItem::Chord(String::from(&line[1..end_idx])));
            }
        }

        // Translation
        if line.starts_with('&') {
            self.index += len;
            return Some(LineIteratorItem::Translation(String::from(line[1..].trim())));
        }

        // Text
        let end_idx = std::cmp::min(line.find("[").unwrap_or(len), line.find("&").unwrap_or(len));
        self.index += end_idx;
        Some(LineIteratorItem::Text(String::from(&line[0..end_idx])))
    }
}

fn str2wp(string: &str) -> WorshipProLine {
    let string = string.trim();
    match string.chars().next() {
        Some('{') => {
            if let Some(colon_idx) = string.find(":") {
                let key = String::from((&string[1..colon_idx]).trim());
                let value = String::from((&string[colon_idx+1..string.len()-1]).trim());
                return WorshipProLine::Directive((key, value));
            }
            WorshipProLine::Empty
        },
        _  => WorshipProLine::TextChordTrans(string.to_string()),
    }
}

fn wp2multilinevec(line: &WorshipProLine) -> Vec<Multiline> {
    match line {
        WorshipProLine::Empty => Vec::new(),
        WorshipProLine::Directive((key, value)) => match key.as_str() {
            "section" => vec!(Multiline::Keyword((*value).clone())),
            _ => Vec::new(),
        },
        WorshipProLine::TextChordTrans(text_chord_trans) => {
            let mut chord = String::new();
            let mut text = String::new();
            let mut translation = String::new();
            let mut new_text_len = 0;
            let mut new_chord_len = 0;
            for item in LineIterator::new(text_chord_trans) {
                match item {
                    LineIteratorItem::Chord(c) => {
                        for _ in 0..(new_text_len-new_chord_len) {
                            chord.push_str(" ");
                        }
                        chord.push_str(&c);
                        new_chord_len = c.chars().count();
                    },
                    LineIteratorItem::Text(t) => {
                        text.push_str(&t);
                        new_text_len = t.chars().count();
                    },
                    LineIteratorItem::Translation(t) => translation = t,
                }
            }

            let mut vec: Vec<Multiline> = Vec::new();
            if chord.len() > 0 {
                vec.push(Multiline::Chord(chord));
            }
            if text.len() > 0 {
                vec.push(Multiline::Text(text));
            }
            if translation.len() > 0 {
                vec.push(Multiline::Translation(translation));
            }
            vec
        },
    }
}

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

struct Transpose<I> where I: Iterator<Item = WorshipProLine> {
    iter: I,
    key_new: i8,
    key_old: Option<i8>,
}
impl<I> Transpose<I> where I: Iterator<Item = WorshipProLine> {
    fn new(iter: I, key: &str) -> Self {
        Self{iter, key_new: chord_to_level(key), key_old: None}
    }

    fn t(&self, line: Option<WorshipProLine>) -> Option<WorshipProLine> {
        let line = line?;
        let key_new = self.key_new;
        let key_old = self.key_old?;
        if let WorshipProLine::TextChordTrans(line) = line {
            let scale = match  key_new {
                0|2|3|5|7|10 => 0,
                _ => 1,
            };
            let mut line_new = String::new();
            for item in LineIterator::new(&line) {
                match item {
                    LineIteratorItem::Chord(c) => {
                        line_new.push_str("[");
                        for item in ChordIterator::new(&c) {
                            match item {
                                ChordIteratorItem::Transposabel(s) => line_new.push_str(SCALE[scale][(chord_to_level(&s)+key_new-key_old).rem_euclid(12) as usize]),
                                ChordIteratorItem::NotTransposable(s) => line_new.push_str(&s),
                            }
                        }
                        line_new.push_str("]");
                    },
                    LineIteratorItem::Text(t) => line_new.push_str(&t),
                    LineIteratorItem::Translation(t) => {
                        line_new.push_str(" & ");
                        line_new.push_str(&t);
                    },
                }
            }
            return Some(WorshipProLine::TextChordTrans(line_new));
        }
        Some(line)
    }
}
impl<I> Iterator for Transpose<I> where I: Iterator<Item = WorshipProLine> {
    type Item = WorshipProLine;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        match &item {
            Some(WorshipProLine::Directive((key, value))) => {
                if key == "key" {
                    self.key_old = Some(chord_to_level(&value))
                }
                item
            }
            Some(WorshipProLine::TextChordTrans(_)) => self.t(item),
            _ => item,
        }
    }
}

trait IntoTranspose: Iterator {
    fn transpose(self, key: &str) -> Transpose<Self> where Self: Sized + Iterator<Item = WorshipProLine> {
        Transpose::new(self, key)
    }
}
impl<I> IntoTranspose for I where I: Sized + Iterator<Item = WorshipProLine> {

}

const SCALE: [[&'static str; 12];2] = [["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"], ["A", "Bb", "Cb", "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab"]];

fn transpose_to(key_old: &str, key_new: &str, line: WorshipProLine) -> WorshipProLine {
    if let WorshipProLine::TextChordTrans(line) = line {
        let key_old = chord_to_level(key_old);
        let key_new = chord_to_level(key_new);
        let scale = match  key_new {
            0|2|3|5|7|10 => 0,
            _ => 1,
        };

        let mut line_new = String::new();

        for item in LineIterator::new(&line) {
            match item {
                LineIteratorItem::Chord(c) => {
                    line_new.push_str("[");
                    for item in ChordIterator::new(&c) {
                        match item {
                            ChordIteratorItem::Transposabel(s) => line_new.push_str(SCALE[scale][(chord_to_level(&s)+key_new-key_old).rem_euclid(12) as usize]),
                            ChordIteratorItem::NotTransposable(s) => line_new.push_str(&s),
                        }
                    }
                    line_new.push_str("]");
                },
                LineIteratorItem::Text(t) => line_new.push_str(&t),
                LineIteratorItem::Translation(t) => {
                    line_new.push_str(" & ");
                    line_new.push_str(&t);
                },
            }
        }

        return WorshipProLine::TextChordTrans(line_new);
    }
    line
}

fn main() {
    let string = "{title: Du hast einen Plan}
{artist: Felix Rollbühler}
{key: D}
{section: Intro}
[D A/C# Bm G]
{section: Verse 1}
[D]Manchmal frag ich [G]mich, muss denn das so [D]sein,
denn ich weiß es [Bm]nicht, mein Verstand ist zu [A]klein.
Im [D]Gebet frag ich [G]dich und ich weiß, du hörst mir [D]zu,
darum frag ich [Em7]dich, was ist dein Plan für [A]mich?
{section: Interlude 1}
[D G D Em7 A]
{section: Chorus}
Du [D]hast einen Plan, du [A]hast einen Plan,
du [Bm]hast einen Plan, mit [G]mir. (2x)
{section: Verse 2}
[D]Herr hilf mir [G]versteh`n, zu hören, wenn du [D]sprichst,
deine Antwort [Bm]kommt, dessen bin ich mir [A]gewiss.
[D]Herr hilf mir zu [G]seh`n, was du mir zeigen [D]willst,
was wir jetzt nicht [Em7]verstehn gibt später einen [A]Sinn.
{section: Chorus}
{section: Interlude 2}
[Bm A/C# Bm G Bm A/C# Bm A]
{section: Bridge}
Ich [Bm]werde warten Herr, [G]warten Herr,
[Em]warten Herr, bis du [A]sprichst.
Ich werd` [Bm]vertrauen Herr, [G]vertrauen Herr,
ver[Em]trauen Herr, auf deinen [A]Plan. (2x)";

//string.lines().map(|line| str2wp(&line)).map(|line| transpose_to("D", "C", line)).map(|line| wp2multilinevec(&line)).flatten().for_each(|line| {
//Transpose::new(string.lines().map(|line| str2wp(&line)).into_iter(), "Eb").map(|line| wp2multilinevec(&line)).flatten().for_each(|line| {
string.lines().map(|line| str2wp(&line)).transpose("Eb").map(|line| wp2multilinevec(&line)).flatten().for_each(|line| {
    match line {
        Multiline::Keyword(keyword) => println!{"\x1b[31;1m{}\x1b[0m", keyword},
        Multiline::Chord(chord) => println!{"\x1b[32;1m  {}\x1b[0m", chord},
        Multiline::Text(text) => println!{"  \x1b[32m{}\x1b[0m", text},
        Multiline::Translation(translation) => println!{"  {}", translation},
    }
});
}
