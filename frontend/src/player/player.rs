use super::{PagesComponent, TableOfContentsComponent};
use gloo_net::http::Request;
use shared::player::{Player, TocItem};
use stylist::Style;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::{use_event_with_window, use_window_size};
use yew_router::prelude::*;
use serde::Deserialize;
use url::Url;
use crate::Route;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Query {
    pub id: Option<String>,
    pub collection: Option<String>,
}

impl Query {
    pub fn api_url(&self) -> String {
        let base = Url::parse("https://example.net").unwrap();
        let mut url = Url::parse("https://example.net/api/player").unwrap();
        {
            let mut query_pairs = url.query_pairs_mut();

            if let Some(id) = &self.id {
                query_pairs.append_pair("id", id);
            }
            if let Some(collection) = &self.collection {
                query_pairs.append_pair("collection", collection);
            }
        }
        base.make_relative(&url).unwrap().to_string()
    }

    pub fn back_route(&self) -> Route {
        if self.collection.is_some() {
            Route::Collections
        } else if self.id.is_some() {
            Route::Songs
        } else {
            Route::NotFound
        }
    }
}

#[function_component]
pub fn PlayerComponent() -> Html {
    let window_dimensions = use_window_size();
    let navigator = use_navigator().unwrap();
    let query = use_location()
        .unwrap()
        .query::<Query>()
        .unwrap_or(Query::default());

    let player = use_state(|| None);
    let active = use_state(|| false);
    let override_key = use_state(|| None);
    {
        let player = player.clone();
        let api_url = query.api_url();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_player: Player = Request::get(&api_url)
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
        let override_key = override_key.clone();
        let back_route = query.back_route();
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
            } else if e.key() == "A" {
                override_key.set(Some(0))
            } else if e.key() == "B" {
                override_key.set(Some(2))
            } else if e.key() == "C" {
                override_key.set(Some(3))
            } else if e.key() == "D" {
                override_key.set(Some(5))
            } else if e.key() == "E" {
                override_key.set(Some(7))
            } else if e.key() == "F" {
                override_key.set(Some(8))
            } else if e.key() == "G" {
                override_key.set(Some(10))
            } else if e.key() == "b" || e.key() == "-" {
                override_key.set(override_key.map(|key| (key + 11) % 12))
            } else if e.key() == "#" || e.key() == "+" {
                override_key.set(override_key.map(|key| (key + 1) % 12))
            } else if e.key() == "r" {
                override_key.set(None)
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
        let back_route = query.back_route();
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
                    override_key={*override_key}
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
