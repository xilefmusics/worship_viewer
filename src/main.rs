mod song;

use song::Key;
use song::Chord;

fn main() {
    let key = Key::from_string("C").unwrap();
    let key_new = key.transpose(-127);
    let chord = Chord::from_string("A#/C", &key);
    println!("Key: {}", key.to_string());
    println!("Chord: {}", chord.to_string(&key_new).unwrap());
}
