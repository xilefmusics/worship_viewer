use stylist::Style;
use worship_viewer_shared::player::PlayerItem;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub item: PlayerItem,
}

#[function_component]
pub fn PageComponent(props: &Props) -> Html {
    match props.item.clone() {
        PlayerItem::Blob(id) => html! {
            <div class={Style::new(include_str!("page.css")).expect("Unwrapping CSS should work!")}>
                <img src={format!("/api/blobs/{}", id)}/>
            </div>
        },
        PlayerItem::Chords(_) => html! {
            "not supported"
        },
    }
}
