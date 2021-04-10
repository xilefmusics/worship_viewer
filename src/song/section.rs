use super::{Key, Line, Textline};

pub struct Section {
    keyword: Option<String>,
    lines: Vec<Line>,
}

impl Section {
    pub fn new(keyword: Option<String>, lines: Vec<&str>, key: &Key) -> Result<Section, String> {
        let lines: Result<Vec<Line>, String> = lines.iter().map(|line| Line::from_string(line, &key)).collect();
        let lines = lines?;
        Ok(Section{keyword, lines})
    }

    pub fn textlines(&self, key: &Key) -> Result<Vec<Textline>, String> {
        let mut textlines: Vec<Textline> = Vec::new();
        if let Some(keyword) = self.keyword.clone() {
            textlines.push(Textline::KEYWORD(keyword));
        }
        for line in &self.lines {
            for textline in line.textlines(key)? {
                textlines.push(textline);
            }
        }
        Ok(textlines)
    }
}


