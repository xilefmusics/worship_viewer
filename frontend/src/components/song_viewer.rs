use shared::song::Song;
use shared::song::{CharPageSet, SimpleChord};
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
    #[prop_or_default]
    pub page: Option<u32>,
}

#[function_component(SongViewer)]
pub fn song_viewer(props: &Props) -> Html {
    let chars_per_row = props.chars_per_row.unwrap_or(80);

    let div_ref = use_node_ref();
    let (width, height) = use_size(div_ref.clone());
    let font_size = (width as f64 / chars_per_row as f64 * 1.662 * 10000.0).floor() / 10000.0;
    let font_style = format!("font-size: {:.2}px;", font_size);
    let chars_per_column = (chars_per_row as f64 / width as f64 * height as f64 / 2.0) as i32;

    let song_content = if chars_per_row > 10 && chars_per_column > 10 {
        props
            .song
            .format_chord_page(
                (chars_per_row as usize) - 8,
                chars_per_column as usize - 4,
                None,
                None,
                props.page.unwrap_or(0),
                &CharPageSet::new()
                    .keyword_prefix("<span class=\"keyword\">")
                    .keyword_suffix("</span>")
                    .chord_prefix("<span class=\"chord\">")
                    .chord_suffix("</span>"),
            )
            .split("\n")
            .map(|line| {
                html! {
                    <p style={font_style.clone()}>
                        {"    "}{Html::from_html_unchecked(line.to_string().into())}
                    </p>
                }
            })
            .collect::<Html>()
    } else {
        html! {}
    };

    let headline = if let Some(artist) = &props.song.data.artist {
        format!("{} ({})", props.song.data.title, artist)
    } else {
        props.song.data.title.clone()
    };
    html! {
        <div
            ref={div_ref}
            class={Style::new(include_str!("song_viewer.css")).expect("Unwrapping CSS should work!")}
        >
            <p style={font_style.clone()}>{ "    " }</p>
            <p style={font_style.clone()} class="headline">{"    "}{ headline }</p>
            <p style={font_style.clone()}>{ "    " }</p>
            {song_content}
        </div>
    }
}
