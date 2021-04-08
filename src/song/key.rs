pub struct Key {
    pub level: i8,
}

impl Key {
    const SCALE: [[&'static str; 12];2] = [["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"], ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "Cb"]];
    pub fn from_string(string: &str) -> Result<Key, String> {
        let mut iter = string.chars();
        let main = iter.next();
        let extension = iter.next();
        match iter.next() {
            Some(_) => return Err(format!("\"{}\" is not a valid Key", string)),
            None => {},
        }
        let mut level: i8 = match main {
            Some('C') => 0,
            Some('D') => 2,
            Some('E') => 4,
            Some('F') => 5,
            Some('G') => 7,
            Some('A') => 9,
            Some('B') => 11,
            _ => return Err(format!("{} is not a valid Key", string)),
        };
        level += match extension {
            Some('b') => -1,
            Some('#') => 1,
            None => 0,
            _ => return Err(format!("{} is not a valid Key", string)),
        };
        Ok(Key{level})
    }

    pub fn level_to_chord(&self, level:i8) -> String {
        let scale = match self.level {
            1|3|5|6|8|10 => 1,
            _ => 0,
        };
        String::from(Key::SCALE[scale][(self.level + level).rem_euclid(12) as usize])
    }

    pub fn to_string(&self) -> String {
        self.level_to_chord(0)
    }

    pub fn transpose(&self, halftones: i8) -> Key {
        Key{level: (self.level+halftones).rem_euclid(12)}
    }
}
