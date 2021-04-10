use super::{Meta, Section, Textline};

pub struct Song {
    meta: Meta,
    sections: Vec<Section>,
}

impl Song {
    pub fn from_string(string: &str) -> Result<Song, String> {
        let meta = Meta::from_string(string)?;
        let mut sections : Vec<Section> = vec!();
        let mut keyword: Option<String> = None;
        let mut section_line_index = 0;
        for (line_idx, line) in string.lines().enumerate() {
            if let Some(idx1) = line.find("{") {
                if let Some(idx2) = line.find(":") {
                    if let Some(idx3) = line.find("}") {
                        let arg = line[idx2+1..idx3].trim();
                        if let Some (idx4) = arg.find(" ") {
                            if (&arg[..idx4] == "section") {
                                keyword = Some(String::from(&arg[idx4+1..]));
                                section_line_index = line_idx;
                            }
                        }
                    }
                }
            } else {
                if (line_idx > section_line_index+1) {
                    continue;
                }
                let mut lines: Vec<&str> = vec!();
                for line in string.lines().skip(line_idx) {
                    if let Some(idx1) = line.find("{") {
                        break;
                    }
                    lines.push(line);
                }
                sections.push(Section::new(keyword.clone(), lines, &meta.key)?);
            }
        }
        Ok(Song{meta, sections})
    }

    pub fn transpose(& mut self, halftones: i8) {
        self.meta.key = self.meta.key.transpose(halftones);
    }

    pub fn textlines(&self) -> Result<Vec<Textline>, String> {
        let mut textlines: Vec<Textline> = Vec::new();
        for section in &self.sections {
            for textline in section.textlines(&self.meta.key)? {
                textlines.push(textline);
            }
        }
        Ok(textlines)
    }

}
