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
