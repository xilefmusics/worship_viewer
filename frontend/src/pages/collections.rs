use crate::route::Route;
use gloo_net::http::Request;
use shared::{collection::Collection};
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::use_api;

#[function_component(CollectionsPage)]
pub fn collection_page() -> Html {
    let collections = use_state(|| vec![]);
    let user = use_state(|| None);
    let api = use_api();
    {
        let collections = collections.clone();
        let user = user.clone();
        use_effect_with((), move |_| {
            let collections = collections.clone();
            let user = user.clone();
            wasm_bindgen_futures::spawn_local(async move {
                user.set(Some(api.get_users_me().await.unwrap()));
                let fetched_collections: Vec<Collection> = Request::get("/api/collections")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                collections.set(fetched_collections);
            });
            || ()
        });
    };

    let navigator = use_navigator().unwrap();

    let collections = collections
        .iter()
        .map(|collection| {
            let cover = "/api/blobs/".to_string() + &collection.cover;
            let title = &collection.title;
            let onclick = {
                let navigator = navigator.clone();
                let id = collection.id.clone().unwrap();
                move |_: MouseEvent| {
                    navigator
                        .push_with_query(
                            &Route::Player,
                            &([("collection", &id)].iter().cloned().collect::<HashMap<_, _>>()),
                        )
                        .unwrap()
                }
            };
            html! {
                <div
                    class="tile"
                     style={format!("background-image: url('{}'), url('data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22300%22%20height%3D%22200%22%3E%3Crect%20width%3D%22100%25%22%20height%3D%22100%25%22%20fill%3D%22%23ccc%22%2F%3E%3Ctext%20x%3D%2250%25%22%20y%3D%2250%25%22%20dominant-baseline%3D%22middle%22%20text-anchor%3D%22middle%22%20font-size%3D%2220%22%20fill%3D%22%23000%22%3E{}%3C%2Ftext%3E%3C%2Fsvg%3E');", cover, title)}
                    onclick={onclick}
                ></div>
            }
        })
        .collect::<Html>();

    html! {
        <div class={Style::new(include_str!("collections.css")).expect("Unwrapping CSS should work!")}>
            <div class="collections">
                <p>{user.as_ref().map(|user| user.email.as_str()).unwrap_or("loading")}</p>
            </div>
            <div class="collections">
                {collections}
            </div>
        </div>
    }
}
