use crate::components::SongViewer;
use shared::player::PlayerItem;
use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub item: PlayerItem,
    pub font_size: i32,
    pub override_key: Option<u8>,
}

// TODO: Make this component obsolete by adding the blob renderer to the song viewer
#[function_component(PageComponent)]
pub fn page_components(props: &Props) -> Html {
    match &props.item {
        PlayerItem::Blob(id) => html! {
            <div class={Style::new(include_str!("page.css")).expect("Unwrapping CSS should work!")}>
                <img src={format!("/api/blobs/{}", id)}/>
            </div>
        },
        PlayerItem::Chords(song) => {
            html! {
                <SongViewer song={song.clone()}/>
            }
        }
    }
}
