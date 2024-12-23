use gloo::console::log;
use shared::song::Song;
use shared::song::{FormatOutputLines, OutputLine, SimpleChord};
use stylist::Style;
use yew::prelude::*;
use yew_hooks::use_size;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub song: Song,
    #[prop_or_default]
    pub chars_per_row: Option<u32>,
    #[prop_or_default]
    pub override_key: Option<SimpleChord>,
}

#[function_component(SongViewer)]
pub fn song_viewer(props: &Props) -> Html {
    let chars_per_row = props.chars_per_row.unwrap_or(80);

    let div_ref = use_node_ref();
    let (width, height) = use_size(div_ref.clone());
    let font_size = (width as f64 / chars_per_row as f64 * 1.662 * 10000.0).floor() / 10000.0;
    let font_style = format!("font-size: {:.2}px;", font_size);
    let chars_per_column = (chars_per_row as f64 / width as f64 * height as f64) as i32;
    log!("chars_per_column: {}", chars_per_column);

    // let lines = {
    //     let mut lines = Vec::<(String, bool)>::new();
    //     for line in props.song.format_output_lines(props.override_key.clone(), None) {
    //         match line {
    //             OutputLine::Keyword(text) => lines.add((text.to_string(), true)),
    //             OutputLine::Chord(text) => text.to_string(),
    //             OutputLine::Text(text) => text.to_string(),
    //         };
    //     }
    // };

    let line = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGH";
    let lines = vec![line, line];
    html! {
        <div
            ref={div_ref}
            class={Style::new(include_str!("song_viewer.css")).expect("Unwrapping CSS should work!")}
        >
            { for lines.iter().map(|line| html! {
                <p style={font_style.clone()}>{ line }</p>
            })}
        </div>
    }
}
