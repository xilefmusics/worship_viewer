use super::Line;

#[derive(Debug, Clone)]
pub struct ToString<I>
where
    I: Iterator<Item = Line>,
{
    iter: I,
}

impl<I> Iterator for ToString<I>
where
    I: Iterator<Item = Line>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Line::TextChordTrans(text)) => Some(text),
            Some(Line::Directive((key, value))) => Some(format!("{{{}: {}}}", key, value)),
            _ => None,
        }
    }
}

pub trait IntoToString: Iterator {
    fn to_string(self) -> ToString<Self>
    where
        Self: Sized + Iterator<Item = Line>,
    {
        ToString { iter: self }
    }
}

impl<I> IntoToString for I where I: Sized + Iterator<Item = Line> {}
