pub enum ChordIteratorItem {
    Transposabel(String),
    NotTransposable(String),
}

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
