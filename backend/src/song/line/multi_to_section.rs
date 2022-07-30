use super::Section;
use super::{multi, section};

#[derive(Debug, Clone)]
pub struct MultiToSection<I>
where
    I: Iterator<Item = Vec<multi::Line>>,
{
    iter: I,
    lines: Vec<section::Line>,
    keyword: Option<String>,
}

impl<I> MultiToSection<I>
where
    I: Iterator<Item = Vec<multi::Line>>,
{
    pub fn new(iter: I) -> Self {
        let lines: Vec<section::Line> = Vec::new();
        let keyword: Option<String> = None;
        Self {
            iter,
            lines,
            keyword,
        }
    }

    fn pack(&mut self) -> Option<Section> {
        if self.keyword.is_none() && self.lines.len() == 0 {
            return None;
        }

        let result = Section {
            keyword: self.keyword.take(),
            lines: self.lines.clone(),
        };

        self.lines = Vec::new();

        return Some(result);
    }
}

impl<I> Iterator for MultiToSection<I>
where
    I: Iterator<Item = Vec<multi::Line>>,
{
    type Item = section::Section;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(vec) = self.iter.next() {
                // destructure multiline vec
                let mut keyword: Option<String> = None;
                let mut chord: Option<String> = None;
                let mut text: Option<String> = None;
                let mut translation_chord: Option<String> = None;
                let mut translation_text: Option<String> = None;
                let mut comment: Option<String> = None;

                for multiline in vec {
                    match multiline {
                        multi::Keyword(k) => keyword = Some(k),
                        multi::Chord(c) => chord = Some(c),
                        multi::Text(t) => text = Some(t),
                        multi::TranslationChord(c) => translation_chord = Some(c),
                        multi::TranslationText(t) => translation_text = Some(t),
                        multi::Comment(c) => comment = Some(c),
                    }
                }

                // new section
                if keyword.is_some() {
                    if self.keyword.is_some() {
                        let result = self.pack();
                        self.keyword = keyword;
                        return result;
                    } else {
                        self.keyword = keyword;
                    }
                }

                // create line
                if chord.is_some()
                    || text.is_some()
                    || translation_chord.is_some()
                    || translation_text.is_some()
                    || comment.is_some()
                {
                    self.lines.push(section::Line {
                        chord,
                        text,
                        translation_chord,
                        translation_text,
                        comment,
                    })
                }
            } else {
                return self.pack();
            }
        }
    }
}

pub trait IntoMultiToSection: Iterator {
    fn to_section(self) -> MultiToSection<Self>
    where
        Self: Sized + Iterator<Item = Vec<multi::Line>>,
    {
        MultiToSection::new(self)
    }
}

impl<I> IntoMultiToSection for I where I: Sized + Iterator<Item = Vec<multi::Line>> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let iter = std::iter::empty::<Vec<multi::Line>>();
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn empty_vec() {
        let iter = std::iter::once::<Vec<multi::Line>>(Vec::new());
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn only_keyword() {
        let content = vec![multi::Line::Keyword("Keyword".to_string())];
        let iter = std::iter::once::<Vec<multi::Line>>(content);
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(
            vec,
            vec!(Section {
                keyword: Some("Keyword".to_string()),
                lines: Vec::new(),
            })
        );
    }

    #[test]
    fn only_text() {
        let content = vec![multi::Line::Text("Text".to_string())];
        let iter = std::iter::once::<Vec<multi::Line>>(content);
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(
            vec,
            vec!(Section {
                keyword: None,
                lines: vec!(section::Line {
                    chord: None,
                    text: Some("Text".to_string()),
                    translation_text: None,
                    translation_chord: None,
                    comment: None,
                }),
            })
        );
    }

    #[test]
    fn keyword_and_text() {
        let content = vec![
            vec![multi::Line::Keyword("Keyword".to_string())],
            vec![multi::Line::Text("Text".to_string())],
        ];
        let iter = content.into_iter();
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(
            vec,
            vec!(Section {
                keyword: Some("Keyword".to_string()),
                lines: vec!(section::Line {
                    chord: None,
                    text: Some("Text".to_string()),
                    translation_text: None,
                    translation_chord: None,
                    comment: None,
                }),
            })
        );
    }

    #[test]
    fn two_section_with_text() {
        let content = vec![
            vec![multi::Line::Keyword("Keyword1".to_string())],
            vec![multi::Line::Text("Text1".to_string())],
            vec![multi::Line::Keyword("Keyword2".to_string())],
            vec![multi::Line::Text("Text2".to_string())],
        ];
        let iter = content.into_iter();
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(
            vec,
            vec!(
                Section {
                    keyword: Some("Keyword1".to_string()),
                    lines: vec!(section::Line {
                        chord: None,
                        text: Some("Text1".to_string()),
                        translation_text: None,
                        translation_chord: None,
                        comment: None,
                    }),
                },
                Section {
                    keyword: Some("Keyword2".to_string()),
                    lines: vec!(section::Line {
                        chord: None,
                        text: Some("Text2".to_string()),
                        translation_text: None,
                        translation_chord: None,
                        comment: None,
                    }),
                }
            )
        );
    }

    #[test]
    fn full_section() {
        let content = vec![
            vec![multi::Line::Keyword("Keyword".to_string())],
            vec![
                multi::Line::Chord("Chord".to_string()),
                multi::Line::Text("Text".to_string()),
                multi::Line::TranslationText("Translation".to_string()),
            ],
        ];
        let iter = content.into_iter();
        let vec = iter.to_section().collect::<Vec<Section>>();
        assert_eq!(
            vec,
            vec!(Section {
                keyword: Some("Keyword".to_string()),
                lines: vec!(section::Line {
                    chord: Some("Chord".to_string()),
                    text: Some("Text".to_string()),
                    translation_text: Some("Translation".to_string()),
                    translation_chord: None,
                    comment: None,
                }),
            })
        );
    }
}
