mod song;

use song::Key;
use song::Chord;
use song::Line;

fn main() {
    let key = Key::from_string("C").unwrap();
    let key_new = key.transpose(-127);
    let chord = Chord::from_string("A#/C", &key);
    let line = Line::from_string("[G/B]Hello W[Cmay7]orld. & Hallo Welt.", &key).unwrap();
    println!("Key: {}", key.to_string());
    println!("Chord: {}", chord.to_string(&key_new).unwrap());
    println!("Line: {}", line.to_string(&key_new).unwrap());
}
