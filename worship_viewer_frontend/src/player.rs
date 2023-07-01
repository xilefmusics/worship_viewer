use super::player_image::ImagePlayerComponent;
use super::toc::TableOfContentsComponent;
use crate::routes::Route;
use gloo_net::http::Request;
use stylist::Style;
use web_sys::HtmlInputElement;
use worship_viewer_shared::types::PlayerData;
use yew::prelude::*;
use yew_hooks::{use_event_with_window, use_window_size};
use yew_router::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ScrollType {
    #[default]
    OnePage,
    HalfPage,
    TwoPage,
    Book,
}

impl ScrollType {
    pub fn next(&self) -> Self {
        match self {
            Self::OnePage => Self::HalfPage,
            Self::HalfPage => Self::TwoPage,
            Self::TwoPage => Self::Book,
            Self::Book => Self::OnePage,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnePage => "[1]",
            Self::HalfPage => "[1/2]",
            Self::TwoPage => "[2]",
            Self::Book => "[b]",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Index {
    index: usize,
    between_pages: bool,
    max_index: usize,
    scroll_type: ScrollType,
}

impl Index {
    pub fn get(&self) -> usize {
        return self.index;
    }

    pub fn scroll_str(&self) -> &'static str {
        self.scroll_type.to_str()
    }

    pub fn is_between_pages(&self) -> bool {
        self.between_pages
    }

    pub fn is_half_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::HalfPage
    }

    pub fn is_two_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::TwoPage
    }

    pub fn is_book_scroll(&self) -> bool {
        self.scroll_type == ScrollType::Book
    }

    pub fn next_scroll_type(&self) -> Self {
        let mut next = Self {
            index: self.index,
            between_pages: false,
            max_index: self.max_index,
            scroll_type: self.scroll_type.next(),
        };
        if next.scroll_type == ScrollType::Book && next.index % 2 == 0 {
            next = next.prev();
        }
        next
    }

    pub fn set_max_index(&self, max_index: usize) -> Self {
        Self {
            index: 0,
            between_pages: false,
            max_index,
            scroll_type: self.scroll_type.clone(),
        }
    }

    fn increment(&self) -> usize {
        if self.index + 1 < self.max_index {
            self.index + 1
        } else {
            self.index
        }
    }

    fn double_increment(&self) -> usize {
        if self.index + 2 < self.max_index {
            self.index + 2
        } else {
            self.index
        }
    }

    fn decrement(&self) -> usize {
        if self.index > 0 {
            self.index - 1
        } else {
            0
        }
    }

    fn inner_jump(&self, new: usize) -> usize {
        if new >= self.max_index {
            self.max_index - 1
        } else {
            new
        }
    }

    fn double_decrement(&self) -> usize {
        if self.index > 1 {
            self.index - 2
        } else {
            0
        }
    }

    pub fn next(&self) -> Self {
        match self.scroll_type {
            ScrollType::OnePage => Self {
                index: self.increment(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::HalfPage => Self {
                index: if self.between_pages {
                    self.increment()
                } else {
                    self.index
                },
                between_pages: !self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoPage => Self {
                index: self.increment(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::Book => Self {
                index: if self.index % 2 == 1 {
                    self.double_increment()
                } else {
                    self.increment()
                },
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
        }
    }
    pub fn prev(&self) -> Self {
        match self.scroll_type {
            ScrollType::OnePage => Self {
                index: self.decrement(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::HalfPage => Self {
                index: if !self.between_pages {
                    self.decrement()
                } else {
                    self.index
                },
                between_pages: !self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoPage => Self {
                index: self.decrement(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::Book => Self {
                index: if self.index % 2 == 1 {
                    self.double_decrement()
                } else {
                    self.decrement()
                },
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
        }
    }

    pub fn jump(&self, new: usize) -> Self {
        Self {
            index: self.inner_jump(
                if self.scroll_type == ScrollType::Book && new % 2 == 0 && new > 0 {
                    new - 1
                } else {
                    new
                },
            ),
            between_pages: self.between_pages,
            max_index: self.max_index,
            scroll_type: self.scroll_type.clone(),
        }
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
                    index.set(index.set_max_index(fetched_data.data.len()));
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
        use_event_with_window("keydown", move |e: KeyboardEvent| {
            if e.key() == "ArrowDown"
                || e.key() == "ArrowRight"
                || e.key() == " "
                || e.key() == "Enter"
                || e.key() == "j"
            {
                index.set(index.next())
            } else if e.key() == "ArrowUp"
                || e.key() == "ArrowLeft"
                || e.key() == "Backspace"
                || e.key() == "k"
            {
                index.set(index.prev())
            } else if e.key() == "s" {
                index.set(index.next_scroll_type())
            } else if e.key() == "m" {
                active.set(!*active);
            }
        });
    }

    let onclick = {
        let index = index.clone();
        let active = active.clone();
        move |e: MouseEvent| {
            if (e.x() as f64) < window_dimensions.0 * 0.4 {
                index.set(index.prev())
            } else if (e.x() as f64) > window_dimensions.0 * 0.6 {
                index.set(index.next())
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
            index.set(index.jump(value));
        })
    };

    let onclick_back_button = {
        let navigator = navigator.clone();
        move |_: MouseEvent| {
            navigator.push(&Route::Collections);
        }
    };

    if data.is_none() {
        return html! {"Loading"};
    }
    let data = data.as_ref().unwrap().clone();

    let id = data.data[index.get()].clone();

    let id2 = if (index.is_half_page_scroll() && index.is_between_pages()
        || index.is_two_page_scroll()
        || index.is_book_scroll() && index.get() != 0)
        && data.data.len() > index.get() + 1
    {
        Some(data.data[index.get() + 1].clone())
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
                <ImagePlayerComponent
                    id={id}
                    id2={id2}
                    active={*active}
                    half_page_scroll={index.is_half_page_scroll()}
                />
            </div>
            <div class={if *active {"bottom active"} else {"bottom"}}>
                <input
                    type="range"
                    min="0"
                    max={(data.data.len()-1).to_string()}
                    value={(index.get()).to_string()}
                    class="index-chooser"
                    oninput={oninput}
                />
                <span>{format!("{:?} / {:?}",index.get()+1, data.data.len())}</span>
                <span
                    onclick={onclick_scroll_changer}
                    class="scroll-changer"
                >{index.scroll_str()}</span>
            </div>
            <div class={if *active {"toc active"}else{"toc"}}>
                <TableOfContentsComponent
                    list={data.toc.clone()}
                    select={index_jump_callback}
                />
            </div>
        </div>
    }
}
