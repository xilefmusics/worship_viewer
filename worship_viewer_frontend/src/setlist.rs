use crate::navigation_bar::NavigationBarComponent;
use stylist::Style;
use yew::prelude::*;

#[function_component]
pub fn SetlistComponent() -> Html {
    html! {
        <div class={Style::new(include_str!("setlist.css")).expect("Unwrapping CSS should work!")}>
            <div class="setlists">
                <h1>{"Setlists (comming soon)"}</h1>
            </div>
            <NavigationBarComponent
                select_collection=false
                select_song=false
                select_setlist=true
            />
        </div>
    }
}
