use gloo::utils::document;
use std::f64::consts::SQRT_2;
use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub id: String,
    pub id2: Option<String>,
    pub half_page_scroll: bool,
    pub active: bool,
}

#[function_component]
pub fn ImagePlayerComponent(props: &Props) -> Html {
    let id = props.id.clone();
    let id2 = props.id2.clone();
    let half_page_scroll = props.half_page_scroll;

    let image_width = use_state(|| 0);
    let last_props = use_state(|| None);
    {
        let image_width = image_width.clone();
        let id2 = id2.clone();
        let last_props = last_props.clone();
        let props = Some(props.clone());
        use_effect(move || {
            if *last_props == props {
                return;
            }
            last_props.set(props);
            if let Some(element) = document().get_element_by_id("pdf-viewer") {
                let width = if id2.is_some() && !half_page_scroll {
                    element.scroll_height() as f64 / SQRT_2 * 2.0
                } else {
                    element.scroll_height() as f64 / SQRT_2
                };
                image_width.set(width as i32);
            }
        })
    }

    html! {
        <div
            id="pdf-viewer"
            class={Style::new(include_str!("player_image.css")).expect("Unwrapping CSS should work!")}
        >
            <div
                class="image-wrapper"
                style={format!("width: {:?}px; height: 100%;", *image_width)}
            >
                <img
                    class={if half_page_scroll {"collection first half-page-scroll"}else{"collection first"}}
                    src={format!("/api/blobs/{}", id)}
                    alt={id}
                />
                if let Some(id) = id2 {
                    <img
                        class={if half_page_scroll {"collection second half-page-scroll"}else{"collection second"}}
                        src={format!("/api/blobs/{}", id)}
                        alt={id}
                    />
                }
            </div>
        </div>
    }
}
