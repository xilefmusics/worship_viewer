use crate::navigation_bar::NavigationBarComponent;
use crate::routes::Route;
use crate::top_bar::TopBarComponent;
use gloo_net::http::Request;
use stylist::Style;
use worship_viewer_shared::types::Collection;
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
            let cover = "/api/blobs/".to_string() + &collection.clone().cover;
            let onclick = {
                let navigator = navigator.clone();
                let id = collection.id.clone();
                move |_: MouseEvent| {
                    let id = (&id).to_string();
                    navigator.push(&Route::Player { id });
                }
            };
            html! {
                <div
                    class="tile"
                    style={format!("background-image: url('{}');", cover)}
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
