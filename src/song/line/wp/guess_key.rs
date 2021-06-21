use super::{Line, LineIterator, LineIteratorItem, TextChordTrans};

fn guess_key(chords: Vec<String>) -> String {
    // get cnt
    let mut cnt: Vec<usize> = vec![0; 12];
    for chord in chords {
        for level in extract_levels(&chord) {
            cnt[level] += 1;
        }
    }

    // argmax
    let mut cnt = cnt.iter().enumerate().collect::<Vec<(usize, &usize)>>();
    cnt.sort_by(|a, b| a.1.cmp(b.1));
    let most = vec![cnt[11].0, cnt[10].0, cnt[9].0];

    // choose
    let chord_level = {
        if let Some(chord_level) = build_triple(&most) {
            chord_level
        } else {
            most[0]
        }
    };

    // level to String
    match chord_level {
        0 => "A",
        1 => "Bb",
        2 => "B",
        3 => "C",
        4 => "Db",
        5 => "D",
        6 => "Eb",
        7 => "E",
        8 => "F",
        9 => "Gb",
        10 => "G",
        11 => "Ab",
        _ => "",
    }
    .to_string()
}

fn build_triple(most: &Vec<usize>) -> Option<usize> {
    let most = vec![most[0] as isize, most[1] as isize, most[2] as isize];
    if ((most[0] + 7).rem_euclid(12) == most[1] || (most[0] + 7).rem_euclid(12) == most[2])
        && ((most[0] - 7).rem_euclid(12) == most[1] || (most[0] - 7).rem_euclid(12) == most[2])
    {
        Some(most[0] as usize)
    } else if ((most[1] + 7).rem_euclid(12) == most[0] || (most[1] + 7).rem_euclid(12) == most[2])
        && ((most[1] - 7).rem_euclid(12) == most[0] || (most[1] - 7).rem_euclid(12) == most[2])
    {
        Some(most[1] as usize)
    } else if ((most[2] + 7).rem_euclid(12) == most[0] || (most[2] + 7).rem_euclid(12) == most[1])
        && ((most[2] - 7).rem_euclid(12) == most[0] || (most[2] - 7).rem_euclid(12) == most[1])
    {
        Some(most[2] as usize)
    } else {
        None
    }
}

fn extract_levels(chord: &str) -> Vec<usize> {
    let mut levels: Vec<usize> = Vec::new();
    for c in chord.split(" ") {
        let mut chars = c.chars();

        let base_level: isize = match chars.next() {
            Some('A') => 0,
            Some('B') => 2,
            Some('C') => 3,
            Some('D') => 5,
            Some('E') => 7,
            Some('F') => 8,
            Some('G') => 10,
            _ => continue,
        };

        let update1: isize = match chars.next() {
            Some('b') => -1,
            Some('#') => 1,
            Some('m') => 3,
            _ => 0,
        };

        let update2: isize = match chars.next() {
            Some('m') => 3,
            _ => 0,
        };

        levels.push((base_level + update1 + update2).rem_euclid(12) as usize);
    }
    levels
}

pub trait GuessKey {
    fn guess_key(self) -> String
    where
        Self: Sized + Iterator<Item = Line>,
    {
        // make chord array
        let mut chords: Vec<String> = Vec::new();
        for line in self {
            if let TextChordTrans(line) = line {
                for item in LineIterator::new(&line) {
                    if let LineIteratorItem::Chord(chord) = item {
                        chords.push(chord.to_string());
                    }
                }
            }
        }
        guess_key(chords)
    }
}

impl<I> GuessKey for I where I: Sized + Iterator<Item = Line> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_key_most() {
        assert_eq!(guess_key(vec!["D D".to_string()]), "D".to_string());
    }

    #[test]
    fn guess_key_three() {
        assert_eq!(
            guess_key(vec!["D G G G G A A D".to_string()]),
            "D".to_string()
        );
    }

    #[test]
    fn extract_levels_empty() {
        assert_eq!(extract_levels("").len(), 0);
    }

    #[test]
    fn extract_levels_base_levels() {
        assert_eq!(extract_levels("A B C D E F G"), vec![0, 2, 3, 5, 7, 8, 10]);
    }

    #[test]
    fn extract_levels_first_update() {
        assert_eq!(extract_levels("D Db D# Dm"), vec![5, 4, 6, 8]);
    }

    #[test]
    fn extract_levels_second_update() {
        assert_eq!(extract_levels("D Dm Dbm D#m"), vec![5, 8, 7, 9]);
    }

    #[test]
    fn extract_levels_wrap_around() {
        assert_eq!(extract_levels("Ab Gm"), vec![11, 1]);
    }

    #[test]
    fn extract_levels_ignore_rest() {
        assert_eq!(extract_levels("D/F# D2"), vec![5, 5]);
    }

    #[test]
    fn extract_levels_bullshit() {
        assert_eq!(extract_levels(" asjdfläkjas;rf").len(), 0);
    }
}
