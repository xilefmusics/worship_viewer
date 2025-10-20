use fancy_yew::components::input::StringInput;
use gloo_net::http::Request;
use js_sys::Reflect;
use shared::setlist::Setlist;
use shared::song::Song;
use shared::song::{ChordRepresentation, SimpleChord};
use std::collections::HashMap;
use stylist::Style;
use wasm_bindgen::JsValue;
use web_sys::{DragEvent, TouchEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub setlist: Setlist,
    pub onsave: Callback<Setlist>,
    pub onback: Callback<MouseEvent>,
    pub ondelete: Callback<Setlist>,
}

fn chord_from_value(value: &str) -> Option<SimpleChord> {
    match value {
        "A" => Some(SimpleChord::new(0)),
        "Bb" => Some(SimpleChord::new(1)),
        "B" => Some(SimpleChord::new(2)),
        "C" => Some(SimpleChord::new(3)),
        "Db" => Some(SimpleChord::new(4)),
        "D" => Some(SimpleChord::new(5)),
        "Eb" => Some(SimpleChord::new(6)),
        "E" => Some(SimpleChord::new(7)),
        "F" => Some(SimpleChord::new(8)),
        "F#" => Some(SimpleChord::new(9)),
        "G" => Some(SimpleChord::new(10)),
        "Ab" => Some(SimpleChord::new(11)),
        _ => None,
    }
}

fn format_key_label(key: &SimpleChord) -> String {
    SimpleChord::default()
        .format(key, &ChordRepresentation::Default)
        .to_string()
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub key: Option<SimpleChord>,
    pub original_key: Option<String>,
}

fn move_item_to(mut items: Vec<Item>, from_idx: usize, target_idx: usize) -> Vec<Item> {
    if from_idx >= items.len() {
        return items;
    }

    let len = items.len();
    let insert_idx = target_idx.min(len.saturating_sub(1));

    let item = items.remove(from_idx);
    items.insert(insert_idx, item);
    items
}

#[function_component(SetlistEditor)]
pub fn setlist_editor(props: &Props) -> Html {
    let title = use_state(|| props.setlist.title.clone());
    let id = use_state(|| props.setlist.id.clone());

    let songs = use_state(|| vec![]);
    let items = use_state(|| vec![]);
    let search_query = use_state(String::new);
    let drag_index = use_state(|| None::<usize>);
    let drag_over_index = use_state(|| None::<usize>);
    let show_delete_dialog = use_state(|| false);
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
                    .filter_map(|song| {
                        Some((
                            song.id.as_ref()?.clone(),
                            (
                                song.data.title.clone(),
                                song.data.key.as_ref().map(|key| format_key_label(key)),
                            ),
                        ))
                    })
                    .collect::<HashMap<_, _>>();
                songs.set(fetched_songs);
                let build_items = setlist_songs
                    .iter()
                    .map(|link| {
                        let (title, original_key_label) = map
                            .get(&link.id)
                            .cloned()
                            .unwrap_or_else(|| ("unknown".into(), None));
                        let key = link
                            .key
                            .clone()
                            .or_else(|| original_key_label.as_deref().and_then(chord_from_value));
                        Item {
                            id: link.id.clone(),
                            title,
                            key,
                            original_key: original_key_label,
                        }
                    })
                    .collect::<Vec<_>>();
                items.set(build_items);
            });
            || ()
        });
    };

    {
        let show_delete_dialog = show_delete_dialog.clone();
        use_effect_with(props.setlist.id.clone(), move |_| {
            show_delete_dialog.set(false);
            || ()
        });
    }

    {
        let drag_index = drag_index.clone();
        let drag_over_index = drag_over_index.clone();
        use_effect_with((*items).len(), move |_| {
            drag_index.set(None);
            drag_over_index.set(None);
            || ()
        });
    }

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

    let total_items = (*items).len();
    let disable_save = (*title).trim().is_empty();
    let can_delete = (*id).is_some();
    let mut counts = HashMap::<String, usize>::new();
    for item in (*items).iter() {
        *counts.entry(item.id.clone()).or_insert(0) += 1;
    }
    let setlist_counts = counts;
    let search_term = (*search_query).trim().to_lowercase();
    let has_filter = !search_term.is_empty();
    let on_clear_search = {
        let search_query = search_query.clone();
        Callback::from(move |_: MouseEvent| search_query.set(String::new()))
    };
    let filtered_songs = (*songs)
        .iter()
        .filter(|song| {
            if has_filter {
                song.data.title.to_lowercase().contains(&search_term)
            } else {
                true
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    let open_delete_dialog = {
        let show_delete_dialog = show_delete_dialog.clone();
        Callback::from(move |_: MouseEvent| show_delete_dialog.set(true))
    };
    let close_delete_dialog = {
        let show_delete_dialog = show_delete_dialog.clone();
        Callback::from(move |_: MouseEvent| show_delete_dialog.set(false))
    };
    let confirm_delete = {
        let items = items.clone();
        let title = title.clone();
        let id = id.clone();
        let show_delete_dialog = show_delete_dialog.clone();
        let ondelete = props.ondelete.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(current_id) = (*id).clone() {
                let setlist = Setlist {
                    id: Some(current_id),
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
                show_delete_dialog.set(false);
                ondelete.emit(setlist);
            } else {
                show_delete_dialog.set(false);
            }
        })
    };
    let stop_dialog_click = Callback::from(|event: MouseEvent| event.stop_propagation());

    html! {
        <div class={Style::new(include_str!("setlist_editor.css")).expect("Unwrapping CSS should work!")}>
            <header class="editor-header">
                <div class="editor-header__group editor-header__group--left">
                    <button
                        type="button"
                        class="icon-button"
                        onclick={props.onback.clone()}
                    >
                        <span class="material-symbols-outlined">{"arrow_back"}</span>
                    </button>
                </div>
                <div class="editor-header__group editor-header__group--right">
                    {
                        if can_delete {
                            html! {
                                <button
                                    type="button"
                                    class="icon-button"
                                    onclick={open_delete_dialog.clone()}
                                >
                                    <span class="material-symbols-outlined">{"delete"}</span>
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <button
                        type="button"
                        class={classes!("primary-button", if disable_save { "primary-button--disabled" } else { "" })}
                        onclick={onsave.clone()}
                        disabled={disable_save}
                    >
                        <span class="material-symbols-outlined">{"save"}</span>
                        <span>{"Save"}</span>
                    </button>
                </div>
            </header>
            {
                if *show_delete_dialog {
                    html! {
                        <div class="dialog-backdrop" onclick={close_delete_dialog.clone()}>
                            <div
                                class="dialog dialog--danger"
                                role="dialog"
                                aria-modal="true"
                                onclick={stop_dialog_click.clone()}
                            >
                                <span class="dialog__title">{"Delete this setlist?"}</span>
                                <p class="dialog__body">
                                    {"This action will permanently remove the setlist. This cannot be undone."}
                                </p>
                                <div class="dialog__actions">
                                    <button
                                        type="button"
                                        class="dialog__button dialog__button--ghost"
                                        onclick={close_delete_dialog.clone()}
                                    >
                                        {"Cancel"}
                                    </button>
                                    <button
                                        type="button"
                                        class="dialog__button dialog__button--danger"
                                        onclick={confirm_delete.clone()}
                                    >
                                        <span class="material-symbols-outlined">{"delete"}</span>
                                        <span>{"Delete setlist"}</span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            <section class="meta">
                <div class="meta__field">
                    <div class="input-shell input-shell--title">
                        <span class="material-symbols-outlined input-shell__icon">{"music_note"}</span>
                        <StringInput
                            bind_handle={title.clone()}
                            placeholder="Give your setlist a name"
                        />
                    </div>
                </div>
                <div class="meta__summary">
                    <span class="meta__count">
                        { format!("{} song{}", total_items, if total_items == 1 { "" } else { "s" }) }
                    </span>
                </div>
            </section>
            <div class="editor-main">
                <section class="panel panel--setlist">
                    <div class="panel__header">
                        <h2>{"Current setlist"}</h2>
                    </div>
                    {
                        if total_items == 0 {
                            html! {
                                <div class="empty-state">
                                    <span class="material-symbols-outlined empty-state__icon">{"playlist_add"}</span>
                                    <p>{"Your setlist is empty."}</p>
                                    <p>{"Add songs from the library to get started."}</p>
                                </div>
                            }
                        } else {
                            html! {
                                <ul class="setlist">
                                    {
                                        for (*items).iter().enumerate().map(|(idx, item)| {
                                            let total_items = total_items;
                                            let current_drag_index = (*drag_index).clone();
                                            let current_drag_over = (*drag_over_index).clone();
                                            let drag_index_handle = drag_index.clone();
                                            let drag_over_handle = drag_over_index.clone();
                                            let can_move_up = idx > 0;
                                            let can_move_down = idx + 1 < total_items;

                                            let onremove = {
                                                let items = items.clone();
                                                move |_: MouseEvent| {
                                                    let mut new_items = (*items).clone();
                                                    if idx < new_items.len() {
                                                        new_items.remove(idx);
                                                        items.set(new_items);
                                                    }
                                                }
                                            };
                                            let on_move_up = {
                                                let items = items.clone();
                                                move |_: MouseEvent| {
                                                    if idx == 0 {
                                                        return;
                                                    }
                                                    let mut new_items = (*items).clone();
                                                    if idx < new_items.len() {
                                                        new_items.swap(idx, idx - 1);
                                                        items.set(new_items);
                                                    }
                                                }
                                            };
                                            let on_move_down = {
                                                let items = items.clone();
                                                move |_: MouseEvent| {
                                                    let mut new_items = (*items).clone();
                                                    if idx + 1 < new_items.len() {
                                                        new_items.swap(idx, idx + 1);
                                                        items.set(new_items);
                                                    }
                                                }
                                            };

                                            let available_keys = ["A", "Bb", "B", "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab"];
                                            let default_key_value = item
                                                .original_key
                                                .as_ref()
                                                .and_then(|key| {
                                                    if available_keys.iter().any(|candidate| candidate == &key.as_str()) {
                                                        Some(key.clone())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .unwrap_or_else(|| available_keys[0].to_string());
                                            let user_override = item
                                                .key
                                                .as_ref()
                                                .map(|key| format_key_label(key));
                                            let select_value = user_override.clone().unwrap_or_else(|| default_key_value.clone());
                                            let onchange = {
                                                let items = items.clone();
                                                let default_key_value = default_key_value.clone();
                                                Callback::from(move |e: Event| {
                                                    if let Some(target) = e.target() {
                                                        let value = Reflect::get(&target, &JsValue::from_str("value"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_else(|| default_key_value.clone());
                                                        let mut new_items = (*items).clone();
                                                        if idx < new_items.len() {
                                                            let new_key = chord_from_value(&value)
                                                                .or_else(|| chord_from_value(&default_key_value));
                                                            new_items[idx].key = new_key;
                                                            items.set(new_items);
                                                        }
                                                    }
                                                })
                                            };

                                            let index = idx;
                                            let drag_index_for_start = drag_index_handle.clone();
                                            let drag_over_for_start = drag_over_handle.clone();
                                            let on_drag_start = Callback::from(move |event: DragEvent| {
                                                drag_index_for_start.set(Some(index));
                                                drag_over_for_start.set(Some(index));
                                                event.stop_propagation();
                                            });
                                            let drag_over_for_over = drag_over_handle.clone();
                                            let on_drag_over = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                drag_over_for_over.set(Some(index));
                                            });
                                            let drag_over_for_leave = drag_over_handle.clone();
                                            let on_drag_leave = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                drag_over_for_leave.set(None);
                                            });
                                            let items_for_drop = items.clone();
                                            let drag_index_for_drop = drag_index_handle.clone();
                                            let drag_over_for_drop = drag_over_handle.clone();
                                            let on_drop = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                if let Some(from_idx) = (*drag_index_for_drop).clone() {
                                                    let new_items = move_item_to((*items_for_drop).clone(), from_idx, index);
                                                    items_for_drop.set(new_items);
                                                }
                                                drag_index_for_drop.set(None);
                                                drag_over_for_drop.set(None);
                                            });
                                            let drag_index_for_end = drag_index_handle.clone();
                                            let drag_over_for_end = drag_over_handle.clone();
                                            let on_drag_end = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                drag_index_for_end.set(None);
                                                drag_over_for_end.set(None);
                                            });
                                            let drag_index_for_touch_start = drag_index_handle.clone();
                                            let drag_over_for_touch_start = drag_over_handle.clone();
                                            let on_touch_start = Callback::from(move |event: TouchEvent| {
                                                drag_index_for_touch_start.set(Some(index));
                                                drag_over_for_touch_start.set(Some(index));
                                                event.prevent_default();
                                                event.stop_propagation();
                                            });
                                            let drag_over_for_touch_move = drag_over_handle.clone();
                                            let on_touch_move = Callback::from(move |event: TouchEvent| {
                                                drag_over_for_touch_move.set(Some(index));
                                                event.prevent_default();
                                                event.stop_propagation();
                                            });
                                            let items_for_touch_end = items.clone();
                                            let drag_index_for_touch_end = drag_index_handle.clone();
                                            let drag_over_for_touch_end = drag_over_handle.clone();
                                            let on_touch_end = Callback::from(move |event: TouchEvent| {
                                                if let Some(from_idx) = (*drag_index_for_touch_end).clone() {
                                                    let new_items = move_item_to((*items_for_touch_end).clone(), from_idx, index);
                                                    items_for_touch_end.set(new_items);
                                                }
                                                drag_index_for_touch_end.set(None);
                                                drag_over_for_touch_end.set(None);
                                                event.prevent_default();
                                                event.stop_propagation();
                                            });
                                            let on_touch_cancel = on_touch_end.clone();
                                            let subtitle = match (item.original_key.as_ref(), user_override.as_ref()) {
                                                (Some(original), Some(current)) if current != original => {
                                                    format!("Original key {original} • Current key {current}")
                                                }
                                                (Some(original), _) => format!("Original key {original}"),
                                                (None, Some(current)) => format!("Current key {current}"),
                                                _ => String::from("No key information"),
                                            };
                                            let item_classes = classes!(
                                                "setlist-item",
                                                if current_drag_index == Some(idx) { Some("setlist-item--dragging") } else { None },
                                                if current_drag_over == Some(idx) { Some("setlist-item--drag-over") } else { None },
                                            );
                                            html! {
                                                <li
                                                    key={format!("{}-{}", item.id, idx)}
                                                    class={item_classes}
                                                    data-index={idx.to_string()}
                                                    draggable="true"
                                                    ondragstart={on_drag_start}
                                                    ondragover={on_drag_over}
                                                    ondragleave={on_drag_leave}
                                                    ondrop={on_drop}
                                                    ondragend={on_drag_end}
                                                    ontouchstart={on_touch_start}
                                                    ontouchmove={on_touch_move}
                                                    ontouchend={on_touch_end}
                                                    ontouchcancel={on_touch_cancel}
                                                >
                                                    <div class="setlist-item__main">
                                                        <button type="button" class="setlist-item__handle" title="Drag to reorder">
                                                            <span class="material-symbols-outlined">{"drag_indicator"}</span>
                                                        </button>
                                                        <span class="setlist-item__index">{ format!("{:02}", idx + 1) }</span>
                                                        <div class="setlist-item__details">
                                                            <span class="setlist-item__title">{ item.title.clone() }</span>
                                                            <span class="setlist-item__subtitle">{ subtitle }</span>
                                                        </div>
                                                    </div>
                                                    <div class="setlist-item__controls">
                                                        <label class="setlist-item__select">
                                                            <select value={select_value.clone()} onchange={onchange.clone()}>
                                                                {
                                                                    available_keys
                                                                        .iter()
                                                                        .map(|option| {
                                                                            html! {
                                                                                <option value={(*option).to_string()}>{option}</option>
                                                                            }
                                                                        })
                                                                        .collect::<Html>()
                                                                }
                                                            </select>
                                                        </label>
                                                        <div class="setlist-item__actions">
                                                            <button type="button" class="icon-button" onclick={on_move_up} disabled={!can_move_up} title="Move up">
                                                                <span class="material-symbols-outlined">{"arrow_upward"}</span>
                                                            </button>
                                                            <button type="button" class="icon-button" onclick={on_move_down} disabled={!can_move_down} title="Move down">
                                                                <span class="material-symbols-outlined">{"arrow_downward"}</span>
                                                            </button>
                                                            <button type="button" class="icon-button icon-button--danger" onclick={onremove} title="Remove">
                                                                <span class="material-symbols-outlined">{"delete"}</span>
                                                            </button>
                                                        </div>
                                                    </div>
                                                </li>
                                            }
                                        })
                                    }
                                    {
                                        if (*drag_index).is_some() {
                                            let target_idx = total_items;
                                            let current_drag_over_end = (*drag_over_index).clone();
                                            let drag_over_for_over = drag_over_index.clone();
                                            let on_drag_over_end = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                drag_over_for_over.set(Some(target_idx));
                                            });
                                            let drag_index_for_drop = drag_index.clone();
                                            let drag_over_for_drop = drag_over_index.clone();
                                            let items_for_drop = items.clone();
                                            let on_drop_end = Callback::from(move |event: DragEvent| {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                if let Some(from_idx) = (*drag_index_for_drop).clone() {
                                                    let new_items = move_item_to((*items_for_drop).clone(), from_idx, target_idx);
                                                    items_for_drop.set(new_items);
                                                }
                                                drag_index_for_drop.set(None);
                                                drag_over_for_drop.set(None);
                                            });
                                            let drag_over_for_touch_move = drag_over_index.clone();
                                            let on_touch_move_end = Callback::from(move |event: TouchEvent| {
                                                drag_over_for_touch_move.set(Some(target_idx));
                                                event.prevent_default();
                                                event.stop_propagation();
                                            });
                                            let drag_index_for_touch_end = drag_index.clone();
                                            let drag_over_for_touch_end = drag_over_index.clone();
                                            let items_for_touch_end = items.clone();
                                            let on_touch_end_end = Callback::from(move |event: TouchEvent| {
                                                if let Some(from_idx) = (*drag_index_for_touch_end).clone() {
                                                    let new_items = move_item_to((*items_for_touch_end).clone(), from_idx, target_idx);
                                                    items_for_touch_end.set(new_items);
                                                }
                                                drag_index_for_touch_end.set(None);
                                                drag_over_for_touch_end.set(None);
                                                event.prevent_default();
                                                event.stop_propagation();
                                            });
                                            let on_touch_cancel_end = on_touch_end_end.clone();
                                            html! {
                                                <li
                                                    class={classes!(
                                                        "setlist-dropzone",
                                                        "setlist-dropzone--end",
                                                        if current_drag_over_end == Some(target_idx) { "setlist-dropzone--active" } else { "" },
                                                    )}
                                                    data-index={target_idx.to_string()}
                                                    ondragover={on_drag_over_end}
                                                    ondrop={on_drop_end}
                                                    ontouchmove={on_touch_move_end}
                                                    ontouchend={on_touch_end_end}
                                                    ontouchcancel={on_touch_cancel_end}
                                                >
                                                    <span>{"Drop at end"}</span>
                                                </li>
                                            }
                                        } else {
                                            Html::default()
                                        }
                                    }
                                </ul>
                            }
                        }
                    }
                </section>
                <section class="panel panel--library">
                    <div class="panel__header panel__header--library">
                        <h2>{"Song library"}</h2>
                        <div class="panel__search">
                            <div class="input-shell input-shell--search">
                                <span class="material-symbols-outlined input-shell__icon">{"search"}</span>
                                <StringInput
                                    bind_handle={search_query.clone()}
                                    placeholder="Search songs"
                                />
                            </div>
                            {
                                if has_filter {
                                    html! {
                                        <button
                                            type="button"
                                            class="icon-button"
                                            onclick={on_clear_search.clone()}
                                            title="Clear search"
                                        >
                                            <span class="material-symbols-outlined">{"close"}</span>
                                        </button>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    </div>
                    {
                        if filtered_songs.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <span class="material-symbols-outlined empty-state__icon">{"search"}</span>
                                    <p>{"No songs match your search."}</p>
                                </div>
                            }
                        } else {
                            html! {
                                <ul class="song-list">
                                    {
                                        for filtered_songs.iter().map(|song| {
                                            let id = song.id.clone().unwrap_or_default();
                                            let song_title = song.data.title.clone();
                                            let song_key_label = song
                                                .data
                                                .key
                                                .as_ref()
                                                .map(|key| format_key_label(key));
                                            let key = song_key_label.clone().unwrap_or_else(|| "—".into());
                                            let occurrences = setlist_counts.get(&id).cloned().unwrap_or(0);
                                            let already_added = occurrences > 0;
                                            let items_handle = items.clone();
                                            let song_key_label_clone = song_key_label.clone();
                                            let id_for_callback = id.clone();
                                            let title_for_callback = song_title.clone();
                                            let tag_text = if occurrences == 0 {
                                                "Add".to_string()
                                            } else if occurrences == 1 {
                                                "Add again".to_string()
                                            } else {
                                                format!("Add again ({})", occurrences)
                                            };
                                            let onclick = Callback::from(move |_: MouseEvent| {
                                                let mut new_items = (*items_handle).clone();
                                                new_items.push(Item {
                                                    id: id_for_callback.clone(),
                                                    title: title_for_callback.clone(),
                                                    key: song_key_label_clone
                                                        .as_ref()
                                                        .and_then(|label| chord_from_value(label)),
                                                    original_key: song_key_label_clone.clone(),
                                                });
                                                items_handle.set(new_items);
                                            });
                                            html! {
                                                <li class={classes!("song-list__item", if already_added { "song-list__item--added" } else { "" })}>
                                                    <button
                                                        type="button"
                                                       class="song-list__button"
                                                       onclick={onclick}
                                                   >
                                                       <div class="song-list__info">
                                                            <span class="song-list__title">{song_title.clone()}</span>
                                                            <span class="song-list__key">{ format!("Key {}", key) }</span>
                                                        </div>
                                                        <span class="song-list__tag">{tag_text.clone()}</span>
                                                    </button>
                                                </li>
                                            }
                                        })
                                    }
                                </ul>
                            }
                        }
                    }
                </section>
            </div>
        </div>
    }
}
