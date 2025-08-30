use crate::route::Route;
use shared::setlist::Setlist;
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;

#[function_component(SetlistsPage)]
pub fn setlists_page() -> Html {
let setlists = use_state(|| vec![]);
    {
        let setlists = setlists.clone();
        use_effect_with((), move |_| {
            let setlists = setlists.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut fetched_setlists: Vec<Setlist> = Request::get("/api/setlists")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                fetched_setlists.sort_by_key(|setlist| setlist.title.clone());
                setlists.set(fetched_setlists);
            });
            || ()
        });
    };

    let navigator = use_navigator().unwrap();

    let setlists = setlists
        .iter()
        .map(|setlist| {
            let title = setlist.title.clone();
            let onclick = {
                let navigator = navigator.clone();
                let id = setlist.id.clone().unwrap();
                move |_: MouseEvent| {
                    navigator
                        .push_with_query(
                            &Route::Player,
                            &([("setlist", &id)]
                                .iter()
                                .cloned()
                                .collect::<HashMap<_, _>>()),
                        )
                        .unwrap()
                }
            };
            html! {
                <div
                    class="setlist"
                    onclick={onclick}
                >
                    <div class="left">{title}</div>
                    <div class="middle"></div>
                    <div class="right"></div>
                </div>
            }
        })
        .collect::<Html>();

    let new_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| navigator.push(&Route::SetlistEditor)
    };

    html! {
        <div class={Style::new(include_str!("setlists.css")).expect("Unwrapping CSS should work!")}>
            <div class="controlls">
                <span
                    class="material-symbols-outlined back-button"
                    onclick={new_button}
                >{"add"}</span>
            </div>
            <div class="setlists">
                {setlists}
            </div>
        </div>
    }
}
