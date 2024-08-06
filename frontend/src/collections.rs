use crate::navigation_bar::NavigationBarComponent;
use crate::routes::Route;
use crate::top_bar::TopBarComponent;
use gloo_net::http::Request;
use shared::collection::Collection;
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
pub fn CollectionsComponent() -> Html {
    let collections = use_state(|| vec![]);
    {
        let collections = collections.clone();
        use_effect_with((), move |_| {
            let collections = collections.clone();
            wasm_bindgen_futures::spawn_local(async move {
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
            <TopBarComponent
                search_placeholder="Search collections..."
            />
            <div class="collections">
                {collections}
            </div>
            <div class="flex-fill"></div>
            <NavigationBarComponent
                select_collection=true
                select_song=false
                select_setlist=false
            />
        </div>
    }
}
