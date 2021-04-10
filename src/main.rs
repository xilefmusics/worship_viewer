mod song;

use song::Song;
use song::Textline;

fn main() {
    let mut song = Song::from_string("{title: Du hast einen Plan}
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
Ich [Bm]werde warten Herr, [G]warten Herr,
[Em]warten Herr, bis du [A]sprichst.
Ich werd` [Bm]vertrauen Herr, [G]vertrauen Herr,
ver[Em]trauen Herr, auf deinen [A]Plan. (2x)").unwrap();

    song.transpose(1);
    for textline in song.textlines().unwrap() {
        match textline {
            Textline::KEYWORD(keyword) => println!{"\x1b[31;1m{}\x1b[0m", keyword},
            Textline::CHORD(chord) => println!{"\x1b[32;1m  {}\x1b[0m", chord},
            Textline::TEXT(text) => println!{"  \x1b[32m{}\x1b[0m", text},
            Textline::TRANSLATION(translation) => println!{"  {}", translation},
        }
    }
}
