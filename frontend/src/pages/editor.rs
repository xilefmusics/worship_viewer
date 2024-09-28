use crate::components::SongEditor;
use gloo_net::http::Request;
use serde::Deserialize;
use shared::song::Song;
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

    let onsave = {
        let song_handle = song.clone();
        Callback::from(move |song: Song| {
            let song_handle = song_handle.clone();
            if song.id.is_some() {
                wasm_bindgen_futures::spawn_local(async move {
                    song_handle.set(Some(
                        Request::put("/api/songs")
                            .json(&vec![song])
                            .unwrap()
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap(),
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
                            .json()
                            .await
                            .unwrap(),
                    ))
                });
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
