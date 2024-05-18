use super::ImageComponent;
use super::TableOfContentsComponent;
use super::{CustomState, State, StateManager};
use crate::routes::Route;
use gloo_net::http::Request;
use stylist::Style;
use web_sys::HtmlInputElement;
use worship_viewer_shared::types::PlayerData;
use yew::prelude::*;
use yew_hooks::{use_event_with_window, use_window_size};
use yew_router::prelude::*;

fn get_back_route(id: &str) -> Route {
    if id.starts_with("collection") {
        Route::Collections
    } else if id.starts_with("song") {
        Route::Songs
    } else if id.starts_with("setlist") {
        Route::Setlists
    } else {
        Route::NotFound
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub id: String,
}

#[function_component]
pub fn PlayerComponent(props: &Props) -> Html {
    let window_dimensions = use_window_size();
    let navigator = use_navigator().unwrap();

    let id = props.id.clone();
    let back_route = get_back_route(&id);

    let state_manager = use_state(|| None);
    let active = use_state(|| false);
    {
        let state_manager = state_manager.clone();
        use_effect_with((), move |_| {
            let state_manager = state_manager.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_data: PlayerData = Request::get(&format!("/api/player/{}", id))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                state_manager.set(Some(StateManager::new(
                    State::new(fetched_data),
                    CustomState::default(),
                )));
            });
            || ()
        });
    };

    {
        let state_manager = state_manager.clone();
        let active = active.clone();
        let navigator = navigator.clone();
        let back_route = back_route.clone();
        use_event_with_window("keydown", move |e: KeyboardEvent| {
            if let Some(target) = e.target() {
                if target.to_string() == "[object HTMLInputElement]" {
                    return;
                }
            }
            if e.key() == "ArrowDown"
                || e.key() == "PageDown"
                || e.key() == "ArrowRight"
                || e.key() == " "
                || e.key() == "Enter"
                || e.key() == "j"
            {
                state_manager.set(
                    state_manager
                        .as_ref()
                        .map(|state_manager| state_manager.next_page()),
                )
            } else if e.key() == "ArrowUp"
                || e.key() == "PageUp"
                || e.key() == "ArrowLeft"
                || e.key() == "Backspace"
                || e.key() == "k"
            {
                state_manager.set(
                    state_manager
                        .as_ref()
                        .map(|state_manager| state_manager.prev_page()),
                )
            } else if e.key() == "s" {
                state_manager.set(
                    state_manager
                        .as_ref()
                        .map(|state_manager| state_manager.next_scroll_type()),
                )
            } else if e.key() == "m" {
                active.set(!*active);
            } else if e.key() == "Escape" {
                navigator.push(&back_route);
            }
        });
    }

    let onclick = {
        let state_manager = state_manager.clone();
        let active = active.clone();
        move |e: MouseEvent| {
            if (e.x() as f64) < window_dimensions.0 * 0.4 {
                state_manager.set(
                    state_manager
                        .as_ref()
                        .map(|state_manager| state_manager.prev_page()),
                )
            } else if (e.x() as f64) > window_dimensions.0 * 0.6 {
                state_manager.set(
                    state_manager
                        .as_ref()
                        .map(|state_manager| state_manager.next_page()),
                )
            } else {
                active.set(!*active);
            }
        }
    };

    let onclick_scroll_changer = {
        let state_manager = state_manager.clone();
        move |_: MouseEvent| {
            state_manager.set(
                state_manager
                    .as_ref()
                    .map(|state_manager| state_manager.next_scroll_type()),
            );
        }
    };

    let onclick_select_changer = {
        let state_manager = state_manager.clone();
        move |_: MouseEvent| {
            state_manager.set(
                state_manager
                    .as_ref()
                    .map(|state_manager| state_manager.next_select_type()),
            );
        }
    };

    let oninput = {
        let state_manager = state_manager.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            state_manager.set(
                state_manager
                    .as_ref()
                    .map(|state_manager| state_manager.jump(input.value_as_number() as usize)),
            );
        }
    };

    let oninput2 = {
        let state_manager = state_manager.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let number = input.value_as_number() as usize;
            if number < 1 {
                return;
            }
            state_manager.set(
                state_manager
                    .as_ref()
                    .map(|state_manager| state_manager.jump(number - 1)),
            );
        }
    };

    let index_jump_callback = {
        let state_manager = state_manager.clone();
        Callback::from(move |value| {
            state_manager.set(
                state_manager
                    .as_ref()
                    .map(|state_manager| state_manager.jump_page(value)),
            );
        })
    };

    let onclick_back_button = {
        let navigator = navigator.clone();
        let back_route = back_route.clone();
        move |_: MouseEvent| {
            navigator.push(&back_route);
        }
    };

    if state_manager.is_none() {
        return html! {};
    }
    let state_manager = state_manager.as_ref().unwrap();

    html! {
        <div
            class={Style::new(include_str!("player.css")).expect("Unwrapping CSS should work!")}
            >
            <div class={if *active {"top active"} else {"top"}}>
                <span
                    class="material-symbols-outlined back-button"
                    onclick={onclick_back_button}
                >{"arrow_back"}</span>
            </div>
            <div onclick={onclick} class={if *active {"middle active"} else {"middle"}}>
                <ImageComponent
                    id={state_manager.get_blob()}
                    id2={state_manager.get_next_blob()}
                    active={*active}
                    half_page_scroll={state_manager.is_half_page_scroll()}
                />
            </div>
            <div class={if *active {"bottom active"} else {"bottom"}}>
                <span
                    onclick={onclick_select_changer}
                    class="select-changer"
                >{state_manager.get_select_str()}</span>
                <input
                    type="range"
                    min="0"
                    max={state_manager.get_max_index().to_string()}
                    value={state_manager.get_index().to_string()}
                    class="index-chooser"
                    oninput={oninput.clone()}
                />
                <span>
                    <input
                        type="number"
                        min="1"
                        max={state_manager.get_max_string()}
                        value={state_manager.get_string()}
                        class="index-chooser-2"
                        oninput={oninput2}
                    />
                    {format!(" / {}",state_manager.get_max_string())}</span>
                <span
                    onclick={onclick_scroll_changer}
                    class="scroll-changer"
                >{state_manager.get_scroll_str()}</span>
            </div>
            <div class={if *active && state_manager.get_toc_len() > 1 {"toc active"}else{"toc"}}>
                <TableOfContentsComponent
                    list={state_manager.get_toc()}
                    select={index_jump_callback}
                />
            </div>
        </div>
    }
}
