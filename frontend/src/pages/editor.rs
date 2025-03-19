use crate::components::SongEditor;
use crate::route::Route;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::song::Song;
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
            let navigator = navigator.clone();
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
                    let song = Request::post("/api/songs")
                        .json(&vec![song])
                        .unwrap()
                        .send()
                        .await
                        .unwrap()
                        .json::<Vec<Song>>()
                        .await
                        .unwrap()
                        .remove(0);
                    if let Some(id) = song.id.as_ref() {
                        navigator
                            .replace_with_query(
                                &Route::Editor,
                                &([("id", id)].iter().cloned().collect::<HashMap<_, _>>()),
                            )
                            .unwrap()
                    }
                    song_handle.set(Some(song));
                });
            }
        })
    };

    let onimport = {
        let song_handle = song.clone();
        Callback::from(move |url: String| {
            if url.len() == 0 {
                song_handle.set(Some(Song::default()));
                return;
            }

            let url = Url::parse(&url).unwrap();
            let api_url = format!(
                "/api/import/{}{}",
                url.host_str().unwrap_or("unknown").replace(".", "/"),
                url.path()
            );
            let song_handle = song_handle.clone();
            wasm_bindgen_futures::spawn_local(async move {
                song_handle.set(Some(
                    Request::get(&api_url)
                        .send()
                        .await
                        .unwrap()
                        .json::<Song>()
                        .await
                        .unwrap(),
                ))
            });
        })
    };

    let navigator = use_navigator().unwrap();
    let onback = Callback::from(move |_: MouseEvent| {
        navigator.back();
    });

    if song.is_none() {
        return html! {};
    }
    let song = song.as_ref().unwrap().clone();

    html! {
        <div class={Style::new(include_str!("editor.css")).expect("Unwrapping CSS should work!")}>
            <SongEditor
                song={song}
                onsave={onsave}
                onback={onback}
                onimport={onimport}
            />
        </div>
    }
}
