use crate::routes::Route;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub search_placeholder: String,
}

#[function_component]
pub fn TopBarComponent(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let search_placeholder = props.search_placeholder.clone();

    html! {
        <div class={Style::new(include_str!("top_bar.css")).expect("Unwrapping CSS should work!")}>
            <div class="left"></div>
            <div class="center">
                <input type="text" placeholder={search_placeholder} />
            </div>
            <div class={"right"}>
                <span
                    class="material-symbols-outlined account"
                >{"account_circle"}</span>
            </div>
        </div>
    }
}
