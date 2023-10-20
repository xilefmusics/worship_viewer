use gloo::utils::document;
use gloo_console::log;
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
    let image_height = use_state(|| 0);
    {
        let image_width = image_width.clone();
        let image_height = image_height.clone();
        use_effect(move || {
            if let Some(element) = document().get_element_by_id("pdf-viewer") {
                let width = std::cmp::min(
                    element.scroll_width(),
                    (element.scroll_height() as f64 / SQRT_2) as i32,
                );
                let height = std::cmp::min(
                    element.scroll_height(),
                    (element.scroll_width() as f64 * SQRT_2) as i32,
                );

                if width == *image_width {
                    return;
                }
                image_width.set(width);
                image_height.set(height);
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
                style={format!("width: {:?}px; height: {:?}px;", *image_width, *image_height)}
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
