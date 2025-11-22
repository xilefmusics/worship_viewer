use crate::api::use_api;
use crate::components::{SongEditor, SongSavePayload};
use crate::route::Route;
use serde::Deserialize;
use shared::song::{CreateSong, Song};
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Query {
    pub id: Option<String>,
}

#[derive(Clone, PartialEq, Default)]
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
    let api = use_api();
    {
        let song_handle = song.clone();
        let api = api.clone();
        let query_id = query.id.clone();
        use_effect_with(query_id, move |id| {
            if let Some(id) = id.clone() {
                let song_handle = song_handle.clone();
                let api = api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched = api.get_song(&id).await.unwrap();
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
        let api = api.clone();
        Callback::from(move |payload: SongSavePayload| {
            let navigator = navigator.clone();
            let song_handle = song_handle.clone();
            let data = payload.data.clone();
            if let Some(id) = payload.id.clone() {
                let api = api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let updated = api.update_song(&id, &data).await.unwrap();
                    song_handle.set(Some(updated.into()));
                });
            } else {
                if data.data.title.is_empty() {
                    return;
                }
                let api = api.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let created = api.create_song(&data).await.unwrap();
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
        let api = api.clone();
        Callback::from(move |url: String| {
            if url.is_empty() {
                song_handle.set(Some(EditorState::new()));
                return;
            }

            let song_handle = song_handle.clone();
            let url = url.clone();
            let api = api.clone();
            wasm_bindgen_futures::spawn_local(async move {
                song_handle.set(Some(EditorState {
                    id: song_handle
                        .as_ref()
                        .map(|s| s.id.clone())
                        .unwrap_or_default(),
                    data: api.import_song_ultimate_guitar(&url).await.unwrap().into(),
                }));
            });
        })
    };

    let ondelete = {
        let song_handle = song.clone();
        let navigator = navigator.clone();
        let api = api.clone();
        Callback::from(move |target_id: String| {
            let navigator = navigator.clone();
            let song_handle = song_handle.clone();
            let api = api.clone();
            wasm_bindgen_futures::spawn_local(async move {
                api.delete_song(&target_id).await.unwrap();
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
