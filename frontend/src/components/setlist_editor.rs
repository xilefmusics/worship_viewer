use fancy_yew::components::input::StringInput;
use gloo_net::http::Request;
use shared::setlist::Setlist;
use shared::song::Song;
use shared::song::{ChordRepresentation, SimpleChord};
use stylist::Style;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub setlist: Setlist,
    pub onsave: Callback<Setlist>,
    pub onback: Callback<MouseEvent>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub key: Option<SimpleChord>,
}

#[function_component(SetlistEditor)]
pub fn setlist_editor(props: &Props) -> Html {
    let title = use_state(|| props.setlist.title.clone());
    let id = use_state(|| props.setlist.id.clone());

    let songs = use_state(|| vec![]);
    let items = use_state(|| vec![]);
    {
        let songs = songs.clone();
        let setlist_songs = props.setlist.songs.clone();
        let items = items.clone();
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
                let map = fetched_songs
                    .iter()
                    .map(|song| (song.id.as_ref().unwrap().clone(), song.data.title.clone()))
                    .collect::<std::collections::HashMap<_, _>>();
                songs.set(fetched_songs);
                let build_items = setlist_songs
                    .iter()
                    .map(|link| {
                        let title = map.get(&link.id).cloned().unwrap_or("unknown".into());
                        Item {
                            id: link.id.clone(),
                            title,
                            key: link.key.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                items.set(build_items);
            });
            || ()
        });
    };

    let onsave = {
        let items = items.clone();
        let title = title.clone();
        let id = id.clone();
        let onsave_upstream = props.onsave.clone();
        Callback::from(move |_: MouseEvent| {
            let new_setlist = Setlist {
                id: (*id).clone(),
                title: (*title).clone(),
                songs: (*items)
                    .iter()
                    .map(|item| shared::song::Link {
                        id: item.id.clone(),
                        nr: None,
                        key: item.key.clone(),
                    })
                    .collect(),
            };
            onsave_upstream.emit(new_setlist);
        })
    };

    html! {
        <div class={Style::new(include_str!("setlist_editor.css")).expect("Unwrapping CSS should work!")}>
            <div class="editor-header">
                <span
                    class="material-symbols-outlined button"
                    onclick={props.onback.clone()}
                >{"arrow_back"}</span>
                <span
                    class="material-symbols-outlined button"
                    onclick={onsave}
                >{"save"}</span>
            </div>
            <div class="meta">
                <StringInput
                    bind_handle={title}
                    placeholder="Setlist Title"
                />
            </div>
            <div class="editor-main">
                <ul class="setlist">
                    {
                        for (*items).iter().enumerate().map(|(idx, item)| {
                            let ondelete = {
                                let items = items.clone();
                                move |_: MouseEvent| {
                                    let mut new_items = (*items).clone();
                                    new_items.remove(idx);
                                    items.set(new_items);
                                }
                            };
                            let onchange = {
                                let items = items.clone();
                                move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    let key = if input.value() == "A" {
                                        Some(SimpleChord::new(0))
                                    } else if input.value() == "Bb" {
                                        Some(SimpleChord::new(1))
                                    } else if input.value() == "B" {
                                        Some(SimpleChord::new(2))
                                    } else if input.value() == "C" {
                                        Some(SimpleChord::new(3))
                                    } else if input.value() == "Db" {
                                        Some(SimpleChord::new(4))
                                    } else if input.value() == "D" {
                                        Some(SimpleChord::new(5))
                                    } else if input.value() == "Eb" {
                                        Some(SimpleChord::new(6))
                                    } else if input.value() == "E" {
                                        Some(SimpleChord::new(7))
                                    } else if input.value() == "F" {
                                        Some(SimpleChord::new(8))
                                    } else if input.value() == "F#" {
                                        Some(SimpleChord::new(9))
                                    } else if input.value() == "G" {
                                        Some(SimpleChord::new(10))
                                    } else if input.value() == "Ab" {
                                        Some(SimpleChord::new(11))
                                    } else {
                                        None
                                    };
                                    let mut new_items = (*items).clone();
                                    new_items[idx].key = key;
                                    items.set(new_items);
                                }
                            };
                            html!{
                                <li>
                                    <div class="left">
                                        <span>{item.title.clone()}</span>
                                    </div>
                                    <div class="middle">
                                    </div>
                                    <div class="right">
                                        <select onchange={onchange}>
                                            {
                                                vec!["default", "A", "Bb", "B", "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab"]
                                                    .iter()
                                                    .map(|option| html! {
                                                        <option
                                                            value={&**option}
                                                            selected={ *option == item.key.as_ref().map(|key| SimpleChord::default().format(&key, &ChordRepresentation::Default).as_ref()).unwrap_or("default") }>
                                                            {option}
                                                        </option>})
                                                    .collect::<Html>()
                                            }
                                        </select>
                                        <span
                                            class="material-symbols-outlined button"
                                            onclick={ondelete}
                                        >{"delete"}</span>
                                    </div>
                                </li>
                            }
                        })
                    }
                </ul>
                <ul class="song-list">
                    {
                        for (*songs).iter().map(|song| {
                            let key = song
                                .data
                                .key
                                .as_ref()
                                .map(|key| key.format(&SimpleChord::default(), &ChordRepresentation::Default))
                                .unwrap_or("");
                            let onclick = {
                                let id = song.id.as_ref().unwrap().clone();
                                let title = song.data.title.clone();
                                let items = items.clone();
                                move |_: MouseEvent| {
                                    let item = Item {
                                        id: id.clone(),
                                        title: title.clone(),
                                        key: None,
                                    };
                                    let mut new_items = (*items).clone();
                                    new_items.push(item);
                                    items.set(new_items);
                                }
                            };
                            html!{
                                <li onclick={onclick}>
                                    <span>{song.data.title.clone()}</span>
                                    <span>{" ("}</span>
                                    <span>{key}</span>
                                    <span>{")"}</span>
                                </li>
                            }
                        })
                    }
                </ul>
            </div>
        </div>
    }
}
