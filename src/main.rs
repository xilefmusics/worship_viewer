mod song;

use song::Key;
use song::Chord;
use song::Line;
use song::Song;

fn main() {
    let key = Key::from_string("C").unwrap();
    let key_new = key.transpose(-127);
    let chord = Chord::from_string("A#/C", &key);
    //let line = Line::from_string("[G/B]Hello W[Cmay7]orld. & Hallo Welt.", &key).unwrap();
    let line = Line::from_string("denn ich weiß es [Bm]nicht, mein Verstand ist zu [A]klein.", &key).unwrap();
    let _song = Song::from_string("{title: Du hast einen Plan}
{artist: Felix Rollbühler}
{key: D}
{meta: section Intro}
[D A/C# Bm G]
{meta: section Verse 1}
[D]Manchmal frag ich [G]mich, muss denn das so [D]sein,
denn ich weiß es [Bm]nicht, mein Verstand ist zu [A]klein.
Im [D]Gebet frag ich [G]dich und ich weiß, du hörst mir [D]zu,
darum frag ich [Em7]dich, was ist dein Plan für [A]mich?
{meta: section Interlude 1}
[D G D Em7 A]
{meta: section Chorus}
Du [D]hast einen Plan, du [A]hast einen Plan,
du [Bm]hast einen Plan, mit [G]mir. (2x)
{meta: section Verse 2}
[D]Herr hilf mir [G]versteh`n, zu hören, wenn du [D]sprichst,
deine Antwort [Bm]kommt, dessen bin ich mir [A]gewiss.
[D]Herr hilf mir zu [G]seh`n, was du mir zeigen [D]willst,
was wir jetzt nicht [Em7]verstehn gibt später einen [A]Sinn.
{meta: section Chorus}
{meta: section Interlude 2}
[Bm A/C# Bm G Bm A/C# Bm A]
{meta: section Bridge}
Ich [Bm]werde warten Herr, [G]warten Berr,
[Em]warten Herr, bis du [A]sprichst.
Ich werd` [Bm]vertrauen Herr, [G]vertrauen Herr,
ver[Em]trauen Herr, auf deinen [A]Plan. (2x)").unwrap();

    println!("Key: {}", key.to_string());
    println!("Chord: {}", chord.to_string(&key_new).unwrap());
    println!("Line: {}", line.to_string(&key_new).unwrap());
}
