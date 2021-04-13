pub enum LineIteratorItem {
    Chord(String),
    Text(String),
    Translation(String),
}

pub struct LineIterator {
    line: String,
    index: usize,
}

impl LineIterator {
    pub fn new(line: &str) -> Self {
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
