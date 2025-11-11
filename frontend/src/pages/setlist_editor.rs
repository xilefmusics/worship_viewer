use crate::components::{SetlistEditor, SetlistSavePayload};
use crate::route::Route;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::setlist::{CreateSetlist, Setlist};
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
        self.id
            .as_ref()
            .map(|id| format!("/api/v1/setlists/{}", id))
    }
}

#[derive(Clone, PartialEq)]
struct EditorState {
    id: Option<String>,
    data: CreateSetlist,
}

impl EditorState {
    fn new() -> Self {
        Self {
            id: None,
            data: CreateSetlist::default(),
        }
    }
}

impl From<Setlist> for EditorState {
    fn from(value: Setlist) -> Self {
        Self {
            id: Some(value.id),
            data: CreateSetlist {
                title: value.title,
                songs: value.songs,
            },
        }
    }
}

#[function_component(SetlistEditorPage)]
pub fn setlist_editor_page() -> Html {
    let query = use_location()
        .unwrap()
        .query::<Query>()
        .unwrap_or(Query::default());

    let setlist = use_state(|| None::<EditorState>);
    {
        let setlist_handle = setlist.clone();
        use_effect_with((), move |_| {
            if let Some(api_url) = query.api_url() {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched: Setlist = Request::get(&api_url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    setlist_handle.set(Some(fetched.into()));
                });
            } else {
                setlist_handle.set(Some(EditorState::new()));
            }
            || ()
        });
    };

    let navigator = use_navigator().unwrap();

    let onsave = {
        let setlist_handle = setlist.clone();
        let navigator = navigator.clone();
        Callback::from(move |payload: SetlistSavePayload| {
            let navigator = navigator.clone();
            let setlist_handle = setlist_handle.clone();
            let data = payload.data.clone();
            if let Some(id) = payload.id.clone() {
                wasm_bindgen_futures::spawn_local(async move {
                    let updated: Setlist = Request::put(&format!("/api/v1/setlists/{}", id))
                        .json(&data)
                        .unwrap()
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    setlist_handle.set(Some(updated.into()));
                });
            } else {
                if data.title.trim().is_empty() {
                    return;
                }
                wasm_bindgen_futures::spawn_local(async move {
                    let created: Setlist = Request::post("/api/v1/setlists")
                        .json(&data)
                        .unwrap()
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    navigator
                        .replace_with_query(
                            &Route::SetlistEditor,
                            &([("id", &created.id)]
                                .iter()
                                .cloned()
                                .collect::<HashMap<_, _>>()),
                        )
                        .unwrap();
                    setlist_handle.set(Some(created.into()));
                });
            }
        })
    };

    let ondelete = {
        let navigator = navigator.clone();
        let setlist_handle = setlist.clone();
        Callback::from(move |target_id: String| {
            let navigator = navigator.clone();
            let setlist_handle = setlist_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Request::delete(&format!("/api/v1/setlists/{}", target_id))
                    .send()
                    .await
                    .unwrap();
                setlist_handle.set(Some(EditorState::new()));
                navigator.push(&Route::Setlists);
            });
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
                setlist={setlist.data.clone()}
                setlist_id={setlist.id.clone()}
                onsave={onsave}
                onback={onback}
                ondelete={ondelete}
            />
        </div>
    }
}
