use crate::routes::Route;
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
        use_effect_with_deps(
            move |_| {
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
            },
            (),
        );
    };

    let navigator = use_navigator().unwrap();

    let collections = collections
        .iter()
        .map(|collection| {
            let cover = "/api/blobs/".to_string() + &collection.clone().cover;
            let title = collection.title.clone();
            let onclick = {
                let navigator = navigator.clone();
                let id = collection.id.clone().unwrap();
                move |_: MouseEvent| {
                    let id = (&id).to_string();
                    navigator.push(&Route::Player { id });
                }
            };
            html! {
                <img
                    class="collection" src={cover} alt={title}
                    onclick={onclick}
                />
            }
        })
        .collect::<Html>();

    html! {
        <div class={Style::new(include_str!("collections.css")).expect("Unwrapping CSS should work!")}>
            <div class="collections">
                {collections}
            </div>
        </div>
    }
}
