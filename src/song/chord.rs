use super::Key;

pub struct Chord {
    levels: Vec<i8>,
    string: String,
}

impl Chord {
    pub fn from_string(string: &str, key: &Key) -> Chord {
        let mut levels: Vec<i8> = vec!();
        let mut chars: Vec<char> = vec!();
        for c in string.chars() {
            match c {
                'C' => {
                    levels.push((0 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'D' => {
                    levels.push((2 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'E' => {
                    levels.push((4 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'F' => {
                    levels.push((5 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'G' => {
                    levels.push((7 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'A' => {
                    levels.push((9 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'B' => {
                    levels.push((11 - key.level()).rem_euclid(12));
                    chars.push('$');
                },
                'b' => match levels.pop() {
                    Some(val) => levels.push(val-1),
                    _ => ()
                },
                '#' => match levels.pop() {
                    Some(val) => levels.push(val+1),
                    _ => ()
                },
                c   => chars.push(c),
            }
        }
        let string: String = chars.into_iter().collect();
        Chord{levels, string}
    }

    pub fn to_string(&self, key: &Key) -> Result<String, String> {
        let mut chars: Vec<char> = vec!();
        let mut level_iter = self.levels.iter();
        for c in self.string.chars() {
            match c {
                '$' => {
                    match level_iter.next() {
                        Some(level) => {
                            for c in key.level_to_chord(*level).chars() {
                                chars.push(c);
                            }
                        },
                        None => return Err(String::from("too few levels")),
                    }
                },
                c => chars.push(c),
            }
        }
        let string: String = chars.into_iter().collect();
        Ok(string)
    }
}
