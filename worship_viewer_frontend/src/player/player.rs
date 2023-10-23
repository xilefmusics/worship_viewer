use super::ImageComponent;
use super::Index;
use super::TableOfContentsComponent;
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

    let index = use_state(|| Index::default());
    let active = use_state(|| false);

    let data = use_state(|| None);
    {
        let data = data.clone();
        let index = index.clone();
        use_effect_with_deps(
            move |_| {
                let data = data.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_data: PlayerData = Request::get(&format!("/api/player/{}", id))
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    index.set(
                        index
                            .set_max_page_index(fetched_data.data.len())
                            .create_number_index_mappings(&fetched_data.toc),
                    );
                    data.set(Some(fetched_data));
                });
                || ()
            },
            (),
        );
    };

    {
        let index = index.clone();
        let active = active.clone();
        let navigator = navigator.clone();
        let back_route = back_route.clone();
        use_event_with_window("keydown", move |e: KeyboardEvent| {
            if e.key() == "ArrowDown"
                || e.key() == "PageDown"
                || e.key() == "ArrowRight"
                || e.key() == " "
                || e.key() == "Enter"
                || e.key() == "j"
            {
                index.set(index.next_page())
            } else if e.key() == "ArrowUp"
                || e.key() == "PageUp"
                || e.key() == "ArrowLeft"
                || e.key() == "Backspace"
                || e.key() == "k"
            {
                index.set(index.prev_page())
            } else if e.key() == "s" {
                index.set(index.next_scroll_type())
            } else if e.key() == "m" {
                active.set(!*active);
            } else if e.key() == "Escape" {
                navigator.push(&back_route);
            }
        });
    }

    let onclick = {
        let index = index.clone();
        let active = active.clone();
        move |e: MouseEvent| {
            if (e.x() as f64) < window_dimensions.0 * 0.4 {
                index.set(index.prev_page())
            } else if (e.x() as f64) > window_dimensions.0 * 0.6 {
                index.set(index.next_page())
            } else {
                active.set(!*active);
            }
        }
    };

    let onclick_scroll_changer = {
        let index = index.clone();
        move |_: MouseEvent| {
            index.set(index.next_scroll_type());
        }
    };

    let onclick_select_changer = {
        let index = index.clone();
        move |_: MouseEvent| {
            index.set(index.next_select_type());
        }
    };

    let oninput = {
        let index = index.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            index.set(index.jump(input.value_as_number() as usize));
        }
    };

    let index_jump_callback = {
        let index = index.clone();
        Callback::from(move |value| {
            index.set(index.jump_page(value));
        })
    };

    let onclick_back_button = {
        let navigator = navigator.clone();
        let back_route = back_route.clone();
        move |_: MouseEvent| {
            navigator.push(&back_route);
        }
    };

    if data.is_none() {
        return html! {};
    }
    let data = data.as_ref().unwrap().clone();

    let id = if index.is_two_half_page_scroll() {
        if index.get_page_index() % 2 == 0
            && data.data.len() > index.get_page_index() + 1
            && index.get_page_index() > 0
        {
            data.data[index.get_page_index() + 1].clone()
        } else {
            data.data[index.get_page_index()].clone()
        }
    } else {
        data.data[index.get_page_index()].clone()
    };

    let id2 = if (index.is_half_page_scroll() && index.is_between_pages()
        || index.is_two_page_scroll()
        || index.is_book_scroll() && index.get_page_index() != 0)
        && data.data.len() > index.get_page_index() + 1
    {
        Some(data.data[index.get_page_index() + 1].clone())
    } else if index.is_two_half_page_scroll() {
        if index.get_page_index() == 0 {
            None
        } else if index.get_page_index() % 2 == 1 && data.data.len() > index.get_page_index() + 1 {
            Some(data.data[index.get_page_index() + 1].clone())
        } else {
            Some(data.data[index.get_page_index()].clone())
        }
    } else {
        None
    };

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
                    id={id}
                    id2={id2}
                    active={*active}
                    half_page_scroll={index.is_half_page_scroll()}
                />
            </div>
            <div class={if *active {"bottom active"} else {"bottom"}}>
                <span
                    onclick={onclick_select_changer}
                    class="select-changer"
                >{index.get_select_str()}</span>
                <input
                    type="range"
                    min="0"
                    max={index.get_max_index().to_string()}
                    value={index.get_index().to_string()}
                    class="index-chooser"
                    oninput={oninput}
                />
                <span>{format!("{} / {}",index.get_string(), index.get_max_string())}</span>
                <span
                    onclick={onclick_scroll_changer}
                    class="scroll-changer"
                >{index.get_scroll_str()}</span>
            </div>
            <div class={if *active && data.toc.len() > 1 {"toc active"}else{"toc"}}>
                <TableOfContentsComponent
                    list={data.toc.clone()}
                    select={index_jump_callback}
                />
            </div>
        </div>
    }
}
