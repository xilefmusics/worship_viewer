use super::{multi, wp};

pub fn wp_to_multi(line: &wp::Line) -> Vec<multi::Line> {
    match line {
        wp::Directive((key, value)) => match key.as_str() {
            "section" => vec![multi::Keyword((*value).trim().to_string())],
            "comment" => vec![multi::Comment((*value).trim().to_string())],
            _ => Vec::new(),
        },
        wp::TextChordTrans(text_chord_trans) => {
            let mut chord = String::new();
            let mut text = String::new();
            let mut translation_chord = String::new();
            let mut translation_text = String::new();
            let mut new_text_len = 0;
            let mut new_chord_len = 0;
            let mut new_translation_text_len = 0;
            let mut new_translation_chord_len = 0;
            for item in wp::LineIterator::new(text_chord_trans) {
                match item {
                    wp::LineIteratorItem::Chord(c) => {
                        let spaces =
                            std::cmp::max(0, new_text_len as i32 - new_chord_len as i32) as usize;
                        for _ in 0..spaces {
                            chord.push_str(" ");
                        }
                        chord.push_str(&c);
                        new_chord_len = c.chars().count();
                    }
                    wp::LineIteratorItem::Text(t) => {
                        text.push_str(&t);
                        new_text_len = t.chars().count();
                    }
                    wp::LineIteratorItem::TranslationChord(c) => {
                        let spaces = std::cmp::max(
                            0,
                            new_translation_text_len as i32 - new_translation_chord_len as i32,
                        ) as usize;
                        for _ in 0..spaces {
                            translation_chord.push_str(" ");
                        }
                        translation_chord.push_str(&c);
                        new_translation_chord_len = c.chars().count();
                    }
                    wp::LineIteratorItem::TranslationText(t) => {
                        translation_text.push_str(&t);
                        new_translation_text_len = t.chars().count();
                    }
                }
            }

            if translation_chord.len() == 0 && text.len() == 0 {
                translation_chord = chord.clone();
            }

            let mut vec: Vec<multi::Line> = Vec::new();
            if chord.len() > 0 {
                vec.push(multi::Chord(chord));
            }
            if text.len() > 0 {
                vec.push(multi::Text(text));
            }
            if translation_chord.len() > 0 {
                vec.push(multi::TranslationChord(translation_chord));
            }
            if translation_text.len() > 0 {
                vec.push(multi::TranslationText(translation_text));
            }
            vec
        }
    }
}

#[derive(Debug, Clone)]
pub struct WpToMulti<I>
where
    I: Iterator<Item = wp::Line>,
{
    iter: I,
}

impl<I> Iterator for WpToMulti<I>
where
    I: Iterator<Item = wp::Line>,
{
    type Item = Vec<multi::Line>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(wp_to_multi(&self.iter.next()?))
    }
}

pub trait IntoWpToMulti: Iterator {
    fn to_multi_flatten(self) -> std::iter::Flatten<WpToMulti<Self>>
    where
        Self: Sized + Iterator<Item = wp::Line>,
    {
        WpToMulti { iter: self }.flatten()
    }
    fn to_multi(self) -> WpToMulti<Self>
    where
        Self: Sized + Iterator<Item = wp::Line>,
    {
        WpToMulti { iter: self }
    }
}

impl<I> IntoWpToMulti for I where I: Sized + Iterator<Item = wp::Line> {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{multi, wp};

    #[test]
    fn empty_text_chord_trans() {
        assert_eq!(
            wp_to_multi(&wp::Line::TextChordTrans("".to_string())).len(),
            0
        );
    }

    #[test]
    fn text() {
        let wp = wp::Line::TextChordTrans("This is a line".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(vec, vec!(multi::Line::Text("This is a line".to_string())));
    }

    #[test]
    fn chord() {
        let wp = wp::Line::TextChordTrans("[Ab/Eb]".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord("Ab/Eb".to_string()),
                multi::Line::TranslationChord("Ab/Eb".to_string())
            )
        );
    }

    #[test]
    fn text_chord() {
        let wp = wp::Line::TextChordTrans("[F6/A]This [G7/B]is a line[C]".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord("F6/A G7/B     C".to_string()),
                multi::Line::Text("This is a line".to_string())
            )
        );
    }

    #[test]
    fn text_chord_translation_asci() {
        let wp = wp::Line::TextChordTrans(
            "[F6/A]This [G7/B]is a line[C] & Das ist eine Zeile".to_string(),
        );
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord("F6/A G7/B     C".to_string()),
                multi::Line::Text("This is a line".to_string()),
                multi::Line::TranslationText("Das ist eine Zeile".to_string())
            )
        );
    }

    #[test]
    fn text_chord_translation_unicode() {
        let wp = wp::Line::TextChordTrans("ß[ä]ö & 'b".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord(" ä".to_string()),
                multi::Line::Text("ßö".to_string()),
                multi::Line::TranslationText("'b".to_string())
            )
        );
    }

    #[test]
    fn text_chord_translation_trailing_whitespaces() {
        let wp = wp::Line::TextChordTrans(" [C]Hello & Hallo ".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord("C".to_string()),
                multi::Line::Text("Hello".to_string()),
                multi::Line::TranslationText("Hallo".to_string())
            )
        );
    }

    #[test]
    fn section() {
        let wp = wp::Line::Directive(("section".to_string(), "Chorus".to_string()));
        let vec = wp_to_multi(&wp);
        assert_eq!(vec, vec!(multi::Line::Keyword("Chorus".to_string())));
    }

    #[test]
    fn section_trailing_whitespaces() {
        let wp = wp::Line::Directive(("section".to_string(), " Chorus ".to_string()));
        let vec = wp_to_multi(&wp);
        assert_eq!(vec, vec!(multi::Line::Keyword("Chorus".to_string())));
    }

    #[test]
    fn section_unicode() {
        let wp = wp::Line::Directive(("section".to_string(), "ä'ö".to_string()));
        let vec = wp_to_multi(&wp);
        assert_eq!(vec, vec!(multi::Line::Keyword("ä'ö".to_string())));
    }

    #[test]
    fn other_directive() {
        let wp = wp::Line::Directive(("other".to_string(), "don't care".to_string()));
        let vec = wp_to_multi(&wp);
        assert_eq!(vec, vec!());
    }

    #[test]
    fn comment() {
        let wp = wp::Line::Directive(("comment".to_string(), "content of comment".to_string()));
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(multi::Line::Comment("content of comment".to_string()))
        );
    }

    #[test]
    fn full_translation() {
        let wp = wp::Line::TextChordTrans("Hel[C]lo World & Hallo [C]Welt".to_string());
        let vec = wp_to_multi(&wp);
        assert_eq!(
            vec,
            vec!(
                multi::Line::Chord("   C".to_string()),
                multi::Line::Text("Hello World".to_string()),
                multi::Line::TranslationChord("      C".to_string()),
                multi::Line::TranslationText("Hallo Welt".to_string())
            )
        );
    }
}
