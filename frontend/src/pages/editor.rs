use crate::components::{SongEditor, SongSavePayload};
use crate::route::Route;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::song::{CreateSong, Song};
use std::collections::HashMap;
use stylist::Style;
use url::Url;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Query {
    pub id: Option<String>,
}

impl Query {
    pub fn api_url(&self) -> Option<String> {
        self.id.as_ref().map(|id| format!("/api/v1/songs/{}", id))
    }
}

#[derive(Clone, PartialEq)]
struct EditorState {
    id: Option<String>,
    data: CreateSong,
}

impl EditorState {
    fn new() -> Self {
        Self {
            id: None,
            data: CreateSong::default(),
        }
    }
}

impl From<Song> for EditorState {
    fn from(value: Song) -> Self {
        let id = if value.id.is_empty() {
            None
        } else {
            Some(value.id.clone())
        };
        Self {
            id,
            data: value.into(),
        }
    }
}

#[function_component(EditorPage)]
pub fn editor_page() -> Html {
    let query = use_location()
        .unwrap()
        .query::<Query>()
        .unwrap_or(Query::default());

    let song = use_state(|| None::<EditorState>);
    {
        let song_handle = song.clone();
        use_effect_with((), move |_| {
            if let Some(api_url) = query.api_url() {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched: Song = Request::get(&api_url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    song_handle.set(Some(fetched.into()));
                });
            } else {
                song_handle.set(Some(EditorState::new()));
            }
            || ()
        });
    };

    let navigator = use_navigator().unwrap();
    let onsave = {
        let song_handle = song.clone();
        let navigator = navigator.clone();
        Callback::from(move |payload: SongSavePayload| {
            let navigator = navigator.clone();
            let song_handle = song_handle.clone();
            let data = payload.data.clone();
            if let Some(id) = payload.id.clone() {
                wasm_bindgen_futures::spawn_local(async move {
                    let updated: Song = Request::put(&format!("/api/v1/songs/{}", id))
                        .json(&data)
                        .unwrap()
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    song_handle.set(Some(updated.into()));
                });
            } else {
                if data.data.title.is_empty() {
                    return;
                }
                wasm_bindgen_futures::spawn_local(async move {
                    let created: Song = Request::post("/api/v1/songs")
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
                            &Route::Editor,
                            &([("id", &created.id)]
                                .iter()
                                .cloned()
                                .collect::<HashMap<_, _>>()),
                        )
                        .unwrap();
                    song_handle.set(Some(created.into()));
                });
            }
        })
    };

    let onimport = {
        let song_handle = song.clone();
        Callback::from(move |url: String| {
            if url.is_empty() {
                song_handle.set(Some(EditorState::new()));
                return;
            }

            let url = Url::parse(&url).unwrap();
            let api_url = format!(
                "/api/import/{}{}",
                url.host_str().unwrap_or("unknown").replace('.', "/"),
                url.path()
            );
            let song_handle = song_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let imported: Song = Request::get(&api_url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                song_handle.set(Some(imported.into()));
            });
        })
    };

    let ondelete = {
        let song_handle = song.clone();
        let navigator = navigator.clone();
        Callback::from(move |target_id: String| {
            let navigator = navigator.clone();
            let song_handle = song_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Request::delete(&format!("/api/v1/songs/{}", target_id))
                    .send()
                    .await
                    .unwrap();
                song_handle.set(Some(EditorState::new()));
                navigator.push(&Route::Songs);
            });
        })
    };

    let onback = {
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.back();
        })
    };

    if song.is_none() {
        return html! {};
    }
    let song = song.as_ref().unwrap().clone();

    html! {
        <div class={Style::new(include_str!("editor.css")).expect("Unwrapping CSS should work!")}>
            <SongEditor
                song={song.data.clone()}
                song_id={song.id.clone()}
                onsave={onsave}
                ondelete={ondelete}
                onback={onback}
                onimport={onimport}
            />
        </div>
    }
}
