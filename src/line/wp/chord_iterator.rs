#[derive(Debug, Clone, PartialEq)]
pub enum ChordIteratorItem<'a> {
    Transposabel(&'a str),
    NotTransposable(&'a str),
}

#[derive(Debug, Clone)]
pub struct ChordIterator<'a> {
    chord: &'a str,
}

impl<'a> ChordIterator<'a> {
    pub fn new(chord: &'a str) -> Self {
        Self { chord }
    }
}

impl<'a> Iterator for ChordIterator<'a> {
    type Item = ChordIteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.chord.chars();

        let transposable = match chars.next() {
            Some(c) => match c {
                'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' => true,
                _ => false,
            },
            None => return None,
        };

        if transposable {
            match chars.next() {
                Some('b') | Some('#') => {
                    let res = &self.chord[..2];
                    self.chord = &self.chord[2..];
                    Some(ChordIteratorItem::Transposabel(res))
                }
                _ => {
                    let res = &self.chord[..1];
                    self.chord = &self.chord[1..];
                    Some(ChordIteratorItem::Transposabel(res))
                }
            }
        } else {
            let mut len = 1;
            while let Some(c) = chars.next() {
                match c {
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' => {
                        let res = &self.chord[..len];
                        self.chord = &self.chord[len..];
                        return Some(ChordIteratorItem::NotTransposable(res));
                    }
                    c => len += c.len_utf8(),
                }
            }
            let res = &self.chord[..];
            self.chord = "";
            Some(ChordIteratorItem::NotTransposable(res))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChordIteratorItem::NotTransposable as N;
    use super::ChordIteratorItem::Transposabel as T;
    use super::*;

    #[test]
    fn asci() {
        let mut iter = ChordIterator::new("Ab/Eb A#m79 ADb");
        assert_eq!(iter.next(), Some(T("Ab")));
        assert_eq!(iter.next(), Some(N("/")));
        assert_eq!(iter.next(), Some(T("Eb")));
        assert_eq!(iter.next(), Some(N(" ")));
        assert_eq!(iter.next(), Some(T("A#")));
        assert_eq!(iter.next(), Some(N("m79 ")));
        assert_eq!(iter.next(), Some(T("A")));
        assert_eq!(iter.next(), Some(T("Db")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unicode() {
        let mut iter = ChordIterator::new(" ?ßF#HäÖA");
        assert_eq!(iter.next(), Some(N(" ?ß")));
        assert_eq!(iter.next(), Some(T("F#")));
        assert_eq!(iter.next(), Some(N("HäÖ")));
        assert_eq!(iter.next(), Some(T("A")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn empty() {
        let mut iter = ChordIterator::new("");
        assert_eq!(iter.next(), None);
    }
}
