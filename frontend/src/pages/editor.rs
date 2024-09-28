use crate::components::SongEditor;
use crate::route::Route;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::song::Song;
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
        self.id.as_ref().map(|id| format!("/api/songs/{}", id))
    }
}

#[function_component(EditorPage)]
pub fn editor_page() -> Html {
    let query = use_location()
        .unwrap()
        .query::<Query>()
        .unwrap_or(Query::default());

    let song = use_state(|| None);
    {
        let song_handle = song.clone();
        use_effect_with((), move |_| {
            if let Some(api_url) = query.api_url() {
                wasm_bindgen_futures::spawn_local(async move {
                    song_handle.set(Some(
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
                song_handle.set(Some(Song::default()));
            }
            || ()
        });
    };

    let navigator = use_navigator().unwrap();
    let onsave = {
        let song_handle = song.clone();
        Callback::from(move |song: Song| {
            let song_handle = song_handle.clone();
            let song_handle2 = song_handle.clone();
            if song.id.is_some() {
                wasm_bindgen_futures::spawn_local(async move {
                    song_handle.set(Some(
                        Request::put("/api/songs")
                            .json(&vec![song])
                            .unwrap()
                            .send()
                            .await
                            .unwrap()
                            .json::<Vec<Song>>()
                            .await
                            .unwrap()
                            .remove(0),
                    ));
                });
            } else {
                if song.data.title == "" {
                    return;
                }
                wasm_bindgen_futures::spawn_local(async move {
                    song_handle.set(Some(
                        Request::post("/api/songs")
                            .json(&vec![song])
                            .unwrap()
                            .send()
                            .await
                            .unwrap()
                            .json::<Vec<Song>>()
                            .await
                            .unwrap()
                            .remove(0),
                    ))
                });
            }
            if let Some(id) = song_handle2.as_ref().unwrap().id.as_ref() {
                navigator
                    .push_with_query(
                        &Route::Editor,
                        &([("id", id)].iter().cloned().collect::<HashMap<_, _>>()),
                    )
                    .unwrap()
            }
        })
    };

    if song.is_none() {
        return html! {};
    }
    let song = song.as_ref().unwrap().clone();

    html! {
        <div class={Style::new(include_str!("editor.css")).expect("Unwrapping CSS should work!")}>
            <SongEditor
                song={song}
                onsave={onsave}
            />
        </div>
    }
}
