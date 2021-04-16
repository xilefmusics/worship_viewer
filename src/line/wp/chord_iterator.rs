#[derive(Debug, Clone, PartialEq)]
pub enum ChordIteratorItem {
    Transposabel(String),
    NotTransposable(String),
}

#[derive(Debug, Clone)]
pub struct ChordIterator {
    chord: String,
    index: usize,
}

impl ChordIterator {
    pub fn new(chord: &str) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::ChordIteratorItem::Transposabel as T;
    use super::ChordIteratorItem::NotTransposable as N;

    #[test]
    fn asci() {
        let mut iter = ChordIterator::new("Ab/Eb A#m79 ADb");
        assert_eq!(iter.next(), Some(T("Ab".to_string())));
        assert_eq!(iter.next(), Some(N("/".to_string())));
        assert_eq!(iter.next(), Some(T("Eb".to_string())));
        assert_eq!(iter.next(), Some(N(" ".to_string())));
        assert_eq!(iter.next(), Some(T("A#".to_string())));
        assert_eq!(iter.next(), Some(N("m79 ".to_string())));
        assert_eq!(iter.next(), Some(T("A".to_string())));
        assert_eq!(iter.next(), Some(T("Db".to_string())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unicode() {
        let mut iter = ChordIterator::new(" ?ßF#HäÖA");
        assert_eq!(iter.next(), Some(N(" ?ß".to_string())));
        assert_eq!(iter.next(), Some(T("F#".to_string())));
        assert_eq!(iter.next(), Some(N("HäÖ".to_string())));
        assert_eq!(iter.next(), Some(T("A".to_string())));
        assert_eq!(iter.next(), None);
    }
}
