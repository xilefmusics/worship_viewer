use super::{wp, multi};

pub fn wp_to_multi(line: &wp::Line) -> Vec<multi::Line> {
    match line {
        wp::Empty => Vec::new(),
        wp::Directive((key, value)) => match key.as_str() {
            "section" => vec!(multi::Keyword((*value).clone())),
            _ => Vec::new(),
        },
        wp::TextChordTrans(text_chord_trans) => {
            let mut chord = String::new();
            let mut text = String::new();
            let mut translation = String::new();
            let mut new_text_len = 0;
            let mut new_chord_len = 0;
            for item in wp::LineIterator::new(text_chord_trans) {
                match item {
                    wp::LineIteratorItem::Chord(c) => {
                        for _ in 0..(new_text_len-new_chord_len) {
                            chord.push_str(" ");
                        }
                        chord.push_str(&c);
                        new_chord_len = c.chars().count();
                    },
                    wp::LineIteratorItem::Text(t) => {
                        text.push_str(&t);
                        new_text_len = t.chars().count();
                    },
                    wp::LineIteratorItem::Translation(t) => translation = t,
                }
            }

            let mut vec: Vec<multi::Line> = Vec::new();
            if chord.len() > 0 {
                vec.push(multi::Chord(chord));
            }
            if text.len() > 0 {
                vec.push(multi::Text(text));
            }
            if translation.len() > 0 {
                vec.push(multi::Translation(translation));
            }
            vec
        },
    }
}

pub struct WpToMulti<I> where I: Iterator<Item = wp::Line> {
    iter: I,
}


impl<I> Iterator for WpToMulti<I> where I: Iterator<Item = wp::Line> {
    type Item = Vec<multi::Line>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(wp_to_multi(&self.iter.next()?))
    }
}

pub trait IntoWpToMulti: Iterator {
    fn to_multi(self) -> std::iter::Flatten<WpToMulti<Self>> where Self: Sized + Iterator<Item = wp::Line> {
        WpToMulti{iter: self}.flatten()
    }
}

impl<I> IntoWpToMulti for I where I: Sized + Iterator<Item = wp::Line> {}
