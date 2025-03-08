use shared::song::Key;
use shared::song::Song;
use stylist::Style;
use yew::prelude::*;
use yew_hooks::use_size;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub song: Song,
    #[prop_or_default]
    pub override_key: Option<Key>,
}

#[function_component(SongViewer)]
pub fn song_viewer(props: &Props) -> Html {
    let div_ref = use_node_ref();
    let (_, height) = use_size(div_ref.clone());
    let scale = height as f32 / 1123.0;
    let override_key = props.override_key.as_ref();
    let (html_string, css_string) = props.song.format_html(override_key, None, Some(scale));
    html! {
        <div
            ref={div_ref}
            class={Style::new(format!("{} {}", css_string, include_str!("song_viewer.css"))).expect("Unwrapping CSS should work!")}
        >
            { Html::from_html_unchecked(html_string.into()) }
        </div>
    }
}
