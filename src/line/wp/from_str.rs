use super::{Line, Directive, TextChordTrans, Empty};

pub fn from_str(string: &str) -> Line {
    let string = string.trim();
    match string.chars().next() {
        Some('{') => {
            if let Some(colon_idx) = string.find(":") {
                let key = String::from((&string[1..colon_idx]).trim());
                let value = String::from((&string[colon_idx+1..string.len()-1]).trim());
                return Directive((key, value));
            }
            Empty
        },
        _  => TextChordTrans(string.to_string()),
    }
}

pub struct FromStr<'a, I> where I: Iterator<Item = &'a str> {
    iter: I
}

impl<'a, I> Iterator for FromStr<'a, I> where I: Iterator<Item = &'a str> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        Some(from_str(&self.iter.next()?))
    }
}

pub trait IntoFromStr<'a>: Iterator {
    fn to_wp<>(self) -> FromStr<'a, Self> where Self: Sized + Iterator<Item = &'a str> {
        FromStr{iter: self}
    }
}

impl<'a, I> IntoFromStr<'a> for I where I: Sized + Iterator<Item = &'a str> {}
