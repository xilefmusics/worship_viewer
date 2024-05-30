use super::{PagesComponent, TableOfContentsComponent};
use crate::routes::Route;
use gloo_net::http::Request;
use stylist::Style;
use web_sys::HtmlInputElement;
use worship_viewer_shared::player::{Player, TocItem};
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

    let player = use_state(|| None);
    let active = use_state(|| false);
    {
        let player = player.clone();
        use_effect_with((), move |_| {
            let player = player.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_player: Player = Request::get(&format!("/api/player/{}", id))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                player.set(Some(fetched_player));
            });
            || ()
        });
    };

    {
        let player = player.clone();
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
                player.set(player.as_ref().map(|player| player.next()))
            } else if e.key() == "ArrowUp"
                || e.key() == "PageUp"
                || e.key() == "ArrowLeft"
                || e.key() == "Backspace"
                || e.key() == "k"
            {
                player.set(player.as_ref().map(|player| player.prev()))
            } else if e.key() == "s" {
                player.set(player.as_ref().map(|player| player.next_scroll_type()))
            } else if e.key() == "m" {
                active.set(!*active);
            } else if e.key() == "Escape" {
                navigator.push(&back_route);
            }
        });
    }

    let onclick = {
        let player = player.clone();
        let active = active.clone();
        move |e: MouseEvent| {
            if (e.x() as f64) < window_dimensions.0 * 0.4 {
                player.set(player.as_ref().map(|player| player.prev()))
            } else if (e.x() as f64) > window_dimensions.0 * 0.6 {
                player.set(player.as_ref().map(|player| player.next()))
            } else {
                active.set(!*active);
            }
        }
    };

    let onclick_scroll_changer = {
        let player = player.clone();
        move |_: MouseEvent| {
            player.set(player.as_ref().map(|player| player.next_scroll_type()));
        }
    };

    let oninput = {
        let player = player.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            player.set(
                player
                    .as_ref()
                    .map(|player| player.jump(input.value_as_number() as usize)),
            );
        }
    };

    let oninput2 = {
        let player = player.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let number = input.value_as_number() as usize;
            if number < 1 {
                return;
            }
            player.set(player.as_ref().map(|player| player.jump(number - 1)));
        }
    };

    let index_jump_callback = {
        let player = player.clone();
        Callback::from(move |value| {
            player.set(player.as_ref().map(|player| player.jump(value)));
        })
    };

    let onclick_back_button = {
        let navigator = navigator.clone();
        let back_route = back_route.clone();
        move |_: MouseEvent| {
            navigator.push(&back_route);
        }
    };

    if player.is_none() {
        return html! {};
    }
    let player = player.as_ref().unwrap();

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
                <PagesComponent
                    item={player.item().0.clone()}
                    item2={player.item().1.map(|item| item.clone())}
                    half_page_scroll={player.is_half_page_scroll()}
                    active={*active}
                />
            </div>
            <div class={if *active {"bottom active"} else {"bottom"}}>
                <input
                    type="range"
                    min="0"
                    max={player.max_index().to_string()}
                    value={player.index().to_string()}
                    class="index-chooser"
                    oninput={oninput.clone()}
                />
                <span>
                    <input
                        type="number"
                        min="1"
                        max={(player.max_index()+1).to_string()}
                        value={(player.index()+1).to_string()}
                        class="index-chooser-2"
                        oninput={oninput2}
                    />
                    {format!(" / {}",(player.max_index()+1).to_string())}</span>
                <span
                    onclick={onclick_scroll_changer}
                    class="scroll-changer"
                >{player.scroll_type_str()}</span>
            </div>
            <div class={if *active && player.toc().len() > 1 {"toc active"}else{"toc"}}>
                <TableOfContentsComponent
                    list={player.toc().iter().map(|item| item.clone()).collect::<Vec<TocItem>>()}
                    select={index_jump_callback}
                />
            </div>
        </div>
    }
}
