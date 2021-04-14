mod line;
use line::{wp_to_multi, IterExtToWp, IterExtTranspose, Multiline};

fn main() {
    let string = "{title: Du hast einen Plan}
{artist: Felix Rollbühler}
{key: D}
{section: Intro}
[D A/C# Bm G]
{section: Verse 1}
[D]Manchmal frag ich [G]mich, muss denn das so [D]sein,
denn ich weiß es [Bm]nicht, mein Verstand ist zu [A]klein.
Im [D]Gebet frag ich [G]dich und ich weiß, du hörst mir [D]zu,
darum frag ich [Em7]dich, was ist dein Plan für [A]mich?
{section: Interlude 1}
[D G D Em7 A]
{section: Chorus}
Du [D]hast einen Plan, du [A]hast einen Plan,
du [Bm]hast einen Plan, mit [G]mir. (2x)
{section: Verse 2}
[D]Herr hilf mir [G]versteh`n, zu hören, wenn du [D]sprichst,
deine Antwort [Bm]kommt, dessen bin ich mir [A]gewiss.
[D]Herr hilf mir zu [G]seh`n, was du mir zeigen [D]willst,
was wir jetzt nicht [Em7]verstehn gibt später einen [A]Sinn.
{section: Chorus}
{section: Interlude 2}
[Bm A/C# Bm G Bm A/C# Bm A]
{section: Bridge}
Ich [Bm]werde warten Herr, [G]warten Herr,
[Em]warten Herr, bis du [A]sprichst.
Ich werd` [Bm]vertrauen Herr, [G]vertrauen Herr,
ver[Em]trauen Herr, auf deinen [A]Plan. (2x)";

//string.lines().map(|line| str_to_wp(&line)).transpose("Eb").map(|line| wp_to_multi(&line)).flatten().for_each(|line| {

string.lines().to_wp().transpose("Eb").map(|line| wp_to_multi(&line)).flatten().for_each(|line| {
    match line {
        Multiline::Keyword(keyword) => println!{"\x1b[31;1m{}\x1b[0m", keyword},
        Multiline::Chord(chord) => println!{"\x1b[32;1m  {}\x1b[0m", chord},
        Multiline::Text(text) => println!{"  \x1b[32m{}\x1b[0m", text},
        Multiline::Translation(translation) => println!{"  {}", translation},
    }
});
}
