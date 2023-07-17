use crate::navigation_bar::NavigationBarComponent;
use crate::routes::Route;
use gloo_net::http::Request;
use stylist::Style;
use worship_viewer_shared::types::Song;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
pub fn SongsComponent() -> Html {
    let songs = use_state(|| vec![]);
    {
        let songs = songs.clone();
        use_effect_with_deps(
            move |_| {
                let songs = songs.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut fetched_songs: Vec<Song> = Request::get("/api/songs")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    fetched_songs.sort_by_key(|song| song.title.clone());
                    let fetched_songs: Vec<Song> = fetched_songs
                        .into_iter()
                        .filter(|song| !song.not_a_song)
                        .collect();
                    songs.set(fetched_songs);
                });
                || ()
            },
            (),
        );
    };

    let navigator = use_navigator().unwrap();

    let songs = songs
        .iter()
        .map(|song| {
            let title = song.title.clone();
            let key = song.key.to_str();
            let onclick = {
                let navigator = navigator.clone();
                let id = song.id.clone().unwrap();
                move |_: MouseEvent| {
                    let id = (&id).to_string();
                    navigator.push(&Route::Player { id });
                }
            };
            html! {
                <tr
                    class="song"
                    onclick={onclick}
                >
                    <td>{"<collection>"}</td>
                    <td class="title-column">{title}</td>
                    <td>{key}</td>
                </tr>
            }
        })
        .collect::<Html>();

    html! {
        <div class={Style::new(include_str!("songs.css")).expect("Unwrapping CSS should work!")}>
            <div class="songs">
                <table>
                    {songs}
                </table>
            </div>
            <NavigationBarComponent
                select_collection=false
                select_song=true
                select_setlist=false
            />
        </div>
    }
}
