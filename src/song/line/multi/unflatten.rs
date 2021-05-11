use super::{Chord, Keyword, Line, Text, Translation};

#[derive(Debug, Clone)]
pub struct Unflatten<I>
where
    I: Iterator<Item = Line>,
{
    iter: I,
    buffer: Option<Vec<Line>>,
    has_text: bool,
    has_chord: bool,
    has_translation: bool,
    has_keyword: bool,
}

impl<I> Unflatten<I>
where
    I: Iterator<Item = Line>,
{
    fn new(iter: I) -> Self {
        let buffer: Option<Vec<Line>> = None;
        let has_text = false;
        let has_chord = false;
        let has_translation = false;
        let has_keyword = false;
        Self {
            iter,
            buffer,
            has_text,
            has_chord,
            has_translation,
            has_keyword,
        }
    }
}

impl<I> Iterator for Unflatten<I>
where
    I: Iterator<Item = Line>,
{
    type Item = Vec<Line>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.iter.next();
            match &line {
                Some(Keyword(_)) => {
                    if self.buffer.is_some() {
                        let result = self.buffer.take();
                        self.buffer = Some(vec![line.expect("matched some")]);
                        self.has_keyword = true;
                        self.has_chord = false;
                        self.has_text = false;
                        self.has_translation = false;
                        return result;
                    } else {
                        return Some(vec![line.expect("matched some")]);
                    }
                }
                Some(Chord(_)) => {
                    if self.has_keyword | self.has_chord {
                        let result = self.buffer.take();
                        self.buffer = Some(vec![line.expect("matched some")]);
                        self.has_chord = true;
                        self.has_text = false;
                        self.has_translation = false;
                        self.has_keyword = false;
                        return result;
                    } else {
                        if let Some(buffer) = &mut self.buffer {
                            buffer.push(line.expect("matched some"));
                        } else {
                            self.buffer = Some(vec![line.expect("matched some")]);
                        }
                        self.has_chord = true;
                    }
                }
                Some(Text(_)) => {
                    if self.has_keyword | self.has_text {
                        let result = self.buffer.take();
                        self.buffer = Some(vec![line.expect("matched some")]);
                        self.has_chord = false;
                        self.has_text = true;
                        self.has_translation = false;
                        self.has_keyword = false;
                        return result;
                    } else {
                        if let Some(buffer) = &mut self.buffer {
                            buffer.push(line.expect("matched some"));
                        } else {
                            self.buffer = Some(vec![line.expect("matched some")]);
                        }
                        self.has_text = true;
                    }
                }
                Some(Translation(_)) => {
                    if self.has_keyword | self.has_translation {
                        let result = self.buffer.take();
                        self.buffer = Some(vec![line.expect("matched some")]);
                        self.has_chord = false;
                        self.has_text = false;
                        self.has_translation = true;
                        self.has_keyword = false;
                        return result;
                    } else {
                        if let Some(buffer) = &mut self.buffer {
                            buffer.push(line.expect("matched some"));
                        } else {
                            self.buffer = Some(vec![line.expect("matched some")]);
                        }
                        self.has_translation = true;
                    }
                }
                None => return self.buffer.take(),
            }
        }
    }
}

pub trait IntoUnflatten: Iterator {
    fn unflatten(self) -> Unflatten<Self>
    where
        Self: Sized + Iterator<Item = Line>,
    {
        Unflatten::new(self)
    }
}

impl<I> IntoUnflatten for I where I: Sized + Iterator<Item = Line> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unflatten_empty() {
        let mut iter = std::iter::empty::<Line>().unflatten();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unflatten_two_keywords() {
        assert_eq!(
            vec![
                Keyword("Keyword 1".to_string()),
                Keyword("Keyword 2".to_string()),
            ]
            .into_iter()
            .unflatten()
            .collect::<Vec<Vec<Line>>>(),
            vec![
                vec![Keyword("Keyword 1".to_string())],
                vec![Keyword("Keyword 2".to_string())],
            ]
        );
    }

    #[test]
    fn unflatten_two_chord() {
        assert_eq!(
            vec![Chord("Chord 1".to_string()), Chord("Chord 2".to_string()),]
                .into_iter()
                .unflatten()
                .collect::<Vec<Vec<Line>>>(),
            vec![
                vec![Chord("Chord 1".to_string())],
                vec![Chord("Chord 2".to_string())],
            ]
        );
    }

    #[test]
    fn unflatten_two_text() {
        assert_eq!(
            vec![Text("Text 1".to_string()), Text("Text 2".to_string()),]
                .into_iter()
                .unflatten()
                .collect::<Vec<Vec<Line>>>(),
            vec![
                vec![Text("Text 1".to_string())],
                vec![Text("Text 2".to_string())],
            ]
        );
    }

    #[test]
    fn unflatten_two_translation() {
        assert_eq!(
            vec![
                Translation("Translation 1".to_string()),
                Translation("Translation 2".to_string()),
            ]
            .into_iter()
            .unflatten()
            .collect::<Vec<Vec<Line>>>(),
            vec![
                vec![Translation("Translation 1".to_string())],
                vec![Translation("Translation 2".to_string())],
            ]
        );
    }

    #[test]
    fn unflatten_two_full_line() {
        assert_eq!(
            vec![
                Chord("Chord".to_string()),
                Text("Text".to_string()),
                Translation("Translation".to_string()),
            ]
            .into_iter()
            .unflatten()
            .collect::<Vec<Vec<Line>>>(),
            vec![vec![
                Chord("Chord".to_string()),
                Text("Text".to_string()),
                Translation("Translation".to_string()),
            ]]
        );
    }

    #[test]
    fn two_chord_and_text() {
        assert_eq!(
            vec![
                Chord("Chord".to_string()),
                Text("Text".to_string()),
                Chord("Chord".to_string()),
                Text("Text".to_string()),
            ]
            .into_iter()
            .unflatten()
            .collect::<Vec<Vec<Line>>>(),
            vec![
                vec![Chord("Chord".to_string()), Text("Text".to_string()),],
                vec![Chord("Chord".to_string()), Text("Text".to_string()),]
            ]
        );
    }
}
