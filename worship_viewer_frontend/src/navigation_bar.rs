use crate::routes::Route;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub select_collection: bool,
    pub select_song: bool,
    pub select_setlist: bool,
}

#[function_component]
pub fn NavigationBarComponent(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let onclick_collection_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| {
            navigator.push(&Route::Collections);
        }
    };

    let onclick_song_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| {
            navigator.push(&Route::Songs);
        }
    };

    let onclick_setlist_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| {
            navigator.push(&Route::Setlists);
        }
    };

    html! {
        <div class={Style::new(include_str!("navigation_bar.css")).expect("Unwrapping CSS should work!")}>
                <span
                    class={if props.select_collection {"material-symbols-outlined collection-button button selected"} else {"material-symbols-outlined collection-button button"}}
                    onclick={onclick_collection_button}
                >{"menu_book"}</span>
                <span
                    class={if props.select_song {"material-symbols-outlined song-button button selected"} else {"material-symbols-outlined song-button button"}}
                    onclick={onclick_song_button}
                >{"library_music"}</span>
                <span
                    class={if props.select_setlist {"material-symbols-outlined setlist-button button selected"} else {"material-symbols-outlined setlist-button button"}}
                    onclick={onclick_setlist_button}
                >{"receipt_long"}</span>
        </div>
    }
}
