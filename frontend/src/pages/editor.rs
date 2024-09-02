use gloo_net::http::Request;
use serde::Deserialize;
use shared::song::Song;
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
        let song = song.clone();
        use_effect_with((), move |_| {
            if let Some(api_url) = query.api_url() {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_song: Song = Request::get(&api_url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    song.set(Some(fetched_song));
                });
            } else {
                song.set(Some(Song::default()));
            }
            || ()
        });
    };

    let title = song
        .as_ref()
        .map(|song| song.data.title.clone())
        .unwrap_or("".into());
    html! {
        <h1>{title}</h1>
    }
}
