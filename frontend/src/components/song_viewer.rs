use yew::prelude::*;
use stylist::Style;

#[function_component(SongViewer)]
pub fn song_viewer() -> Html {
    html! {
        <div class={Style::new(include_str!("song_viewer.css")).expect("Unwrapping CSS should work!")}>
            {"Song Viewer"}
        </div>
    }
}
