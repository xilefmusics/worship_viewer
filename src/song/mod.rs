mod key;
pub use key::Key;

mod chord;
pub use chord::Chord;

mod line;
pub use line::Line;

mod textline;
pub use textline::Textline;

mod meta;
pub use meta::Meta;

mod section;
pub use section::Section;

mod song;
pub use song::Song;

pub fn cfind(string: &str, chr: char) -> Option<usize> {
    for (idx, c) in string.chars().enumerate() {
        if c == chr {
            return Some(idx);
        }
    }
    None
}
