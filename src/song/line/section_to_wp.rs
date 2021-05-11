use super::Section;
use super::{section, wp};

fn chord_lines_to_spaces_and_chords(mut line: &str) -> Vec<(usize, String)> {
    let mut result: Vec<(usize, String)> = Vec::new();
    while line.chars().count() > 0 {
        let mut start_idx = 0;
        let spaces = line
            .char_indices()
            .take_while(|(_, c)| *c == ' ')
            .map(|(idx, _)| start_idx = idx)
            .count();
        if start_idx > 0 {
            start_idx += 1;
        }
        line = &line[start_idx..];
        let end_idx = line.find(' ').unwrap_or(line.len());
        let chord = line[..end_idx].to_string();
        line = &line[end_idx..];
        if chord.len() > 0 {
            result.push((spaces, chord));
        }
    }
    result
}

fn section_line_to_wp(line: section::Line) -> wp::Line {
    let mut s = String::new();

    if let Some(text) = line.text {
        if let Some(chord) = line.chord {
            // text and chord
            let mut idx = 0;
            let mut last_chord_len = 0;
            for (spaces, chord) in chord_lines_to_spaces_and_chords(&chord) {
                let t = text
                    .chars()
                    .skip(idx)
                    .take(spaces + last_chord_len)
                    .collect::<String>();
                s.push_str(&t);
                s.push_str(
                    &std::iter::repeat(' ')
                        .take((spaces + last_chord_len) - t.chars().count())
                        .collect::<String>(),
                );
                s.push('[');
                s.push_str(&chord);
                s.push(']');
                idx += t.chars().count();
                last_chord_len = chord.chars().count();
            }
            s.push_str(&text.chars().skip(idx).collect::<String>());
        } else {
            // only text
            s.push_str(&text);
        }
    } else if let Some(chord) = line.chord {
        // only chord
        s.push('[');
        s.push_str(&chord);
        s.push(']');
    }

    // translation
    if let Some(translation) = line.translation {
        s.push_str(" & ");
        s.push_str(&translation);
    }

    wp::Line::TextChordTrans(s)
}

fn keyword_to_wp(keyword: String) -> wp::Line {
    wp::Line::Directive(("section".to_string(), keyword))
}

pub struct SectionToWp<I>
where
    I: Iterator<Item = Section>,
{
    iter: I,
    line_iter: std::vec::IntoIter<section::Line>,
}

impl<I> SectionToWp<I>
where
    I: Iterator<Item = Section>,
{
    pub fn new(iter: I) -> Self {
        let line_iter = Vec::new().into_iter();
        Self { iter, line_iter }
    }
}

impl<I> Iterator for SectionToWp<I>
where
    I: Iterator<Item = Section>,
{
    type Item = wp::Line;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.line_iter.next() {
                return Some(section_line_to_wp(line));
            } else {
                if let Some(section) = self.iter.next() {
                    let keyword = section.keyword.clone();
                    self.line_iter = section.lines.into_iter();
                    if let Some(keyword) = keyword {
                        return Some(keyword_to_wp(keyword));
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

pub trait IntoSectionToWp: Iterator {
    fn to_wp(self) -> SectionToWp<Self>
    where
        Self: Sized + Iterator<Item = Section>,
    {
        SectionToWp::new(self)
    }
}

impl<I> IntoSectionToWp for I where I: Sized + Iterator<Item = Section> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_lines() {
        let vec = chord_lines_to_spaces_and_chords("F/A     G/B  C");
        assert_eq!(
            vec,
            vec![
                (0, "F/A".to_string()),
                (5, "G/B".to_string()),
                (2, "C".to_string())
            ]
        );
    }

    #[test]
    fn text_only() {
        let chord = None;
        let text = Some("Das ist eine Zeile".to_string());
        let translation = None;
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("Das ist eine Zeile".to_string())
        );
    }

    #[test]
    fn chord_only() {
        let chord = Some("Chord".to_string());
        let text = None;
        let translation = None;
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("[Chord]".to_string())
        );
    }

    #[test]
    fn chord_and_text() {
        let chord = Some("F/A     G/B  C".to_string());
        let text = Some("Das ist eine Zeile".to_string());
        let translation = None;
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("[F/A]Das ist [G/B]eine [C]Zeile".to_string())
        );
    }

    #[test]
    fn outstanding_chord() {
        let chord = Some("     Chord".to_string());
        let text = Some("Text".to_string());
        let translation = None;
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("Text [Chord]".to_string())
        );
    }

    #[test]
    fn chord_text_and_translation() {
        let chord = Some("F/A     G/B  C".to_string());
        let text = Some("Das ist eine Zeile".to_string());
        let translation = Some("This is a line".to_string());
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("[F/A]Das ist [G/B]eine [C]Zeile & This is a line".to_string())
        );
    }

    #[test]
    fn unicode() {
        let chord = Some("F/A     G/B  C".to_string());
        let text = Some("Daß äst eine Zeile".to_string());
        let translation = Some("This is a line".to_string());
        let line = section::Line {
            chord,
            text,
            translation,
        };
        assert_eq!(
            section_line_to_wp(line),
            wp::TextChordTrans("[F/A]Daß äst [G/B]eine [C]Zeile & This is a line".to_string())
        );
    }

    #[test]
    fn empty() {
        let iter = std::iter::empty::<Section>();
        let vec = iter.to_wp().collect::<Vec<wp::Line>>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn empty_section_with_keyword() {
        let keyword = Some("Keyword".to_string());
        let lines: Vec<section::Line> = Vec::new();
        let section = Section { keyword, lines };
        let iter = vec![section].into_iter();
        let vec = iter.to_wp().collect::<Vec<wp::Line>>();
        assert_eq!(
            vec,
            vec![wp::Directive((
                "section".to_string(),
                "Keyword".to_string()
            ))]
        );
    }

    #[test]
    fn empty_section_without_keyword() {
        let keyword = None;
        let lines: Vec<section::Line> = Vec::new();
        let section = Section { keyword, lines };
        let iter = vec![section].into_iter();
        let vec = iter.to_wp().collect::<Vec<wp::Line>>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn section_with_keyword() {
        let keyword = Some("Keyword".to_string());
        let chord = Some("F/A     G/B  C".to_string());
        let text = Some("Das ist eine Zeile".to_string());
        let translation = Some("This is a line".to_string());
        let line = section::Line {
            chord,
            text,
            translation,
        };
        let lines: Vec<section::Line> = vec![line];
        let section = Section { keyword, lines };
        let iter = vec![section].into_iter();
        let vec = iter.to_wp().collect::<Vec<wp::Line>>();
        assert_eq!(
            vec,
            vec![
                wp::Directive(("section".to_string(), "Keyword".to_string())),
                wp::TextChordTrans("[F/A]Das ist [G/B]eine [C]Zeile & This is a line".to_string())
            ]
        );
    }
}
