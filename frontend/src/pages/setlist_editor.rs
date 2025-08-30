use crate::components::SetlistEditor;
use crate::route::Route;
use gloo::console::log;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::setlist::Setlist;
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Query {
    pub id: Option<String>,
}

impl Query {
    pub fn api_url(&self) -> Option<String> {
        self.id.as_ref().map(|id| format!("/api/setlists/{}", id))
    }
}

#[function_component(SetlistEditorPage)]
pub fn setlist_editor_page() -> Html {
    let query = use_location()
        .unwrap()
        .query::<Query>()
        .unwrap_or(Query::default());

    let setlist = use_state(|| None);
    {
        let setlist_handle = setlist.clone();
        use_effect_with((), move |_| {
            if let Some(api_url) = query.api_url() {
                wasm_bindgen_futures::spawn_local(async move {
                    setlist_handle.set(Some(
                        Request::get(&api_url)
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap(),
                    ));
                });
            } else {
                setlist_handle.set(Some(Setlist::default()));
            }
            || ()
        });
    };

    let navigator = use_navigator().unwrap();

    let onsave = {
        let setlist_handle = setlist.clone();
        let navigator = navigator.clone();
        Callback::from(move |setlist: Setlist| {
            let setlist_handle = setlist_handle.clone();
            let navigator = navigator.clone();
            if setlist.id.is_some() {
                wasm_bindgen_futures::spawn_local(async move {
                    setlist_handle.set(Some(
                        Request::put("/api/setlists")
                            .json(&vec![setlist])
                            .unwrap()
                            .send()
                            .await
                            .unwrap()
                            .json::<Vec<Setlist>>()
                            .await
                            .unwrap()
                            .remove(0),
                    ));
                });
            } else {
                if setlist.title == "" {
                    return;
                }
                wasm_bindgen_futures::spawn_local(async move {
                    let setlist = Request::post("/api/setlists")
                        .json(&vec![setlist])
                        .unwrap()
                        .send()
                        .await
                        .unwrap()
                        .json::<Vec<Setlist>>()
                        .await
                        .unwrap()
                        .remove(0);
                    if let Some(id) = setlist.id.as_ref() {
                        navigator
                            .replace_with_query(
                                &Route::SetlistEditor,
                                &([("id", id)].iter().cloned().collect::<HashMap<_, _>>()),
                            )
                            .unwrap()
                    }
                    setlist_handle.set(Some(setlist));
                });
            }
        })
    };

    let onback = Callback::from(move |_: MouseEvent| {
        navigator.back();
    });

    if setlist.is_none() {
        return html! {};
    }
    let setlist = setlist.as_ref().unwrap().clone();

    html! {
        <div class={Style::new(include_str!("setlist_editor.css")).expect("Unwrapping CSS should work!")}>
            <SetlistEditor
                setlist={setlist}
                onsave={onsave}
                onback={onback}
            />
        </div>
    }
}
