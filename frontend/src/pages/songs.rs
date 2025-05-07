use crate::route::Route;
use gloo_net::http::Request;
use shared::song::{ChordRepresentation, SimpleChord, Song};
use std::collections::HashMap;
use stylist::Style;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(SongsPage)]
pub fn songs_page() -> Html {
    let songs = use_state(|| vec![]);
    {
        let songs = songs.clone();
        use_effect_with((), move |_| {
            let songs = songs.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut fetched_songs: Vec<Song> = Request::get("/api/songs")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                fetched_songs.sort_by_key(|song| song.data.title.clone());
                let fetched_songs: Vec<Song> = fetched_songs
                    .into_iter()
                    .filter(|song| !song.not_a_song)
                    .collect();
                songs.set(fetched_songs);
            });
            || ()
        });
    };

    let navigator = use_navigator().unwrap();

    let songs = songs
        .iter()
        .map(|song| {
            let title = song.data.title.clone();
            let key = song
                .data
                .key
                .as_ref()
                .map(|key| key.format(&SimpleChord::default(), &ChordRepresentation::Default))
                .unwrap_or("");
            let onclick = {
                let navigator = navigator.clone();
                let id = song.id.clone().unwrap();
                move |_: MouseEvent| {
                    navigator
                        .push_with_query(
                            &Route::Player,
                            &([("id", &id)].iter().cloned().collect::<HashMap<_, _>>()),
                        )
                        .unwrap()
                }
            };
            html! {
                <div
                    class="song"
                    onclick={onclick}
                >
                    <div class="left">{title}</div>
                    <div class="middle"></div>
                    <div class="right">{key}</div>
                </div>
            }
        })
        .collect::<Html>();

    let new_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| navigator.push(&Route::Editor)
    };

    html! {
        <div class={Style::new(include_str!("songs.css")).expect("Unwrapping CSS should work!")}>
            <div class="controlls">
                <span
                    class="material-symbols-outlined back-button"
                    onclick={new_button}
                >{"add"}</span>
            </div>
            <div class="songs">
                {songs}
            </div>
        </div>
    }
}
