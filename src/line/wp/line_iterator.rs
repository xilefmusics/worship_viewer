#[derive(Debug, Clone, PartialEq)]
pub enum LineIteratorItem<'a> {
    Chord(&'a str),
    Text(&'a str),
    Translation(&'a str),
}

#[derive(Debug, Clone)]
pub struct LineIterator<'a> {
    line: &'a str,
}

impl<'a> LineIterator<'a> {
    pub fn new(line: &'a str) -> Self {
        LineIterator { line: line.trim() }
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = LineIteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.line.len();
        // End
        if len <= 0 {
            return None;
        }

        // Chord
        if self.line.starts_with('[') {
            if let Some(end_idx) = self.line.find("]") {
                let res = &self.line[1..end_idx];
                self.line = &self.line[end_idx + 1..];
                return Some(LineIteratorItem::Chord(res));
            }

            // interprete non-closing Chord as Text
            let end_idx = self.line.find("&").unwrap_or(len);
            let res = &self.line[0..end_idx];
            self.line = &self.line[end_idx..];
            return Some(LineIteratorItem::Text(res));
        }

        // Translation
        if self.line.starts_with('&') {
            let res = &self.line[1..].trim();
            self.line = "";
            return Some(LineIteratorItem::Translation(res));
        }

        // Text
        let end_idx = std::cmp::min(
            self.line.find("[").unwrap_or(len),
            self.line.find("&").unwrap_or(len),
        );
        let mut res = &self.line[..end_idx];
        if let None = self.line.find("[") {
            res = res.trim_end();
        }
        self.line = &self.line[end_idx..];
        if res.len() == 0 {
            return self.next();
        }
        Some(LineIteratorItem::Text(res))
    }
}

#[cfg(test)]
mod tests {
    use super::LineIteratorItem::{Chord, Text, Translation};
    use super::*;

    #[test]
    fn empty() {
        let mut iter = LineIterator::new("");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn chord_only() {
        let mut iter = LineIterator::new("[Abm79/Cb]");
        assert_eq!(iter.next(), Some(Chord("Abm79/Cb")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn text_only() {
        let mut iter = LineIterator::new("Hello World");
        assert_eq!(iter.next(), Some(Text("Hello World")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn asci() {
        let mut iter = LineIterator::new("[F6/A]This [G7/B]is a line[C]");
        assert_eq!(iter.next(), Some(Chord("F6/A")));
        assert_eq!(iter.next(), Some(Text("This ")));
        assert_eq!(iter.next(), Some(Chord("G7/B")));
        assert_eq!(iter.next(), Some(Text("is a line")));
        assert_eq!(iter.next(), Some(Chord("C")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn chord_than_text() {
        let mut iter = LineIterator::new("[F6/A]text");
        assert_eq!(iter.next(), Some(Chord("F6/A")));
        assert_eq!(iter.next(), Some(Text("text")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn translation_withoud_spaces() {
        let mut iter = LineIterator::new("Hello&Hallo");
        assert_eq!(iter.next(), Some(Text("Hello")));
        assert_eq!(iter.next(), Some(Translation("Hallo")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn translation_with_spaces() {
        let mut iter = LineIterator::new("Hello & Hallo");
        assert_eq!(iter.next(), Some(Text("Hello")));
        assert_eq!(iter.next(), Some(Translation("Hallo")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn text_chord_translation_with_spaces() {
        let mut iter = LineIterator::new("[F6/A]This [G7/B]is a line[C] & Das ist eine Zeile");
        assert_eq!(iter.next(), Some(Chord("F6/A")));
        assert_eq!(iter.next(), Some(Text("This ")));
        assert_eq!(iter.next(), Some(Chord("G7/B")));
        assert_eq!(iter.next(), Some(Text("is a line")));
        assert_eq!(iter.next(), Some(Chord("C")));
        assert_eq!(iter.next(), Some(Translation("Das ist eine Zeile")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn trailing_spaces() {
        let mut iter = LineIterator::new(" Hello ");
        assert_eq!(iter.next(), Some(Text("Hello")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unicode() {
        let mut iter = LineIterator::new("ß[ä]&ö");
        assert_eq!(iter.next(), Some(Text("ß")));
        assert_eq!(iter.next(), Some(Chord("ä")));
        assert_eq!(iter.next(), Some(Translation("ö")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn not_closing_chord() {
        let mut iter = LineIterator::new("Before[ChordAfter");
        assert_eq!(iter.next(), Some(Text("Before")));
        assert_eq!(iter.next(), Some(Text("[ChordAfter")));
        assert_eq!(iter.next(), None);
    }
}
