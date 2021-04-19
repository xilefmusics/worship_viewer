use super::{Directive, Line, TextChordTrans};

pub fn from_str(string: &str) -> Line {
    let string = string.trim();
    if let Some(start_idx) = string.find("{") {
        if let Some(colon_idx) = string.find(":") {
            if let Some(end_idx) = string.find("}") {
                if start_idx < colon_idx && colon_idx < end_idx {
                    if start_idx == 0 && end_idx == string.len() - 1 {
                        let key = String::from((&string[start_idx + 1..colon_idx]).trim());
                        let value = String::from((&string[colon_idx + 1..end_idx]).trim());
                        if key.len() > 0 && value.len() > 0 {
                            return Directive((key, value));
                        }
                    }
                }
            }
        }
    }
    TextChordTrans(string.to_string())
}

pub struct FromStr<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    iter: I,
}

impl<'a, I> Iterator for FromStr<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        Some(from_str(&self.iter.next()?))
    }
}

pub trait IntoFromStr<'a>: Iterator {
    fn to_wp(self) -> FromStr<'a, Self>
    where
        Self: Sized + Iterator<Item = &'a str>,
    {
        FromStr { iter: self }
    }
}

impl<'a, I> IntoFromStr<'a> for I where I: Sized + Iterator<Item = &'a str> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let vec = "".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn directive() {
        let vec = "{key:value}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::Directive(("key".to_string(), "value".to_string())))
        );
    }

    #[test]
    fn directive_spaces() {
        let vec = "{ key  :   value    }"
            .lines()
            .to_wp()
            .collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::Directive(("key".to_string(), "value".to_string())))
        );
    }

    #[test]
    fn directive_unicode() {
        let vec = "{äber:v'al}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::Directive(("äber".to_string(), "v'al".to_string())))
        );
    }

    #[test]
    fn asci() {
        let vec = "This is a [C]line & Das ist eine Zeile"
            .lines()
            .to_wp()
            .collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::TextChordTrans(
                "This is a [C]line & Das ist eine Zeile".to_string(),
            ))
        );
    }

    #[test]
    fn asci_spaces() {
        let vec = " Hello ".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("Hello".to_string(),)));
    }

    #[test]
    fn unicode() {
        let vec = "Hällo['ö]".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("Hällo['ö]".to_string(),)));
    }

    #[test]
    fn multiline() {
        let vec = "{title: The Title}
{key: The Key}
This is a line"
            .lines()
            .to_wp()
            .collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(
                Line::Directive(("title".to_string(), "The Title".to_string())),
                Line::Directive(("key".to_string(), "The Key".to_string())),
                Line::TextChordTrans("This is a line".to_string())
            )
        );
    }

    #[test]
    fn non_closing_directive() {
        let vec = "{key:value".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("{key:value".to_string())));
    }

    #[test]
    fn non_seperating_directive() {
        let vec = "{word}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("{word}".to_string())));
    }

    #[test]
    fn non_key_directive() {
        let vec = "{:value}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("{:value}".to_string())));
    }

    #[test]
    fn non_value_directive() {
        let vec = "{key:}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("{key:}".to_string())));
    }

    #[test]
    fn empty_directive() {
        let vec = "{}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans("{}".to_string())));
    }

    #[test]
    fn bullshit_directive() {
        let vec = ":}{".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(vec, vec!(Line::TextChordTrans(":}{".to_string())));
    }

    #[test]
    fn directive_followed() {
        let vec = "{key:value}following"
            .lines()
            .to_wp()
            .collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::TextChordTrans("{key:value}following".to_string()))
        );
    }
    #[test]
    fn directive_following() {
        let vec = "pre{key:value}".lines().to_wp().collect::<Vec<Line>>();
        assert_eq!(
            vec,
            vec!(Line::TextChordTrans("pre{key:value}".to_string()))
        );
    }
}
