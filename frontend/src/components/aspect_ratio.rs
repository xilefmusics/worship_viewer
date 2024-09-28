use std::cmp::min;
use stylist::Style;
use yew::prelude::*;
use yew_hooks::use_window_size;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
    pub left: f64,
    #[prop_or_default]
    pub right: f64,
}

#[function_component(AspectRatio)]
pub fn aspect_ratio(props: &Props) -> Html {
    let combined = props.left + props.right;

    let (parent_width, parent_height) = use_window_size();
    let combined_width = min(
        parent_width as u32,
        (parent_height as f64 * combined) as u32,
    );
    let combined_height = min(
        parent_height as u32,
        (parent_width as f64 / combined) as u32,
    );

    let left_width = (combined_width as f64 * props.left / combined) as u32;
    let combined_width = if props.right > 0. {
        combined_width
    } else {
        parent_width as u32
    };
    let right_width = combined_width - left_width;

    let vertical_margin = (parent_height as u32 - combined_height) / 2;
    let horizontal_margin = (parent_width as u32 - combined_width) / 2;

    let horizontal_round_failure_correction =
        parent_width as u32 - 2 * horizontal_margin - left_width - right_width;

    let style = Style::new(format!(
        r#"
        :root {{
            width: 100%;
            height: 100%;
        }}
        .aspect-ratio-container {{
            width: 100%;
            height: 100%;
        }}
        .first-child {{
            position: absolute;
            left: {}px;
            top: {}px;
            width: {}px;
            height: {}px;
        }}
        .last-child {{
            position: absolute;
            right: {}px;
            top: {}px;
            width: {}px;
            height: {}px;
        }}
    "#,
        horizontal_margin,
        vertical_margin,
        left_width,
        combined_height,
        horizontal_margin + horizontal_round_failure_correction,
        vertical_margin,
        right_width,
        combined_height
    ))
    .expect("Unwrapping CSS should work!");

    let first_child = props.children.iter().next().map(|child| {
        html! {
            <div class="first-child">
                { child.clone() }
            </div>
        }
    });

    let last_child = props.children.iter().last().map(|child| {
        html! {
            <div class="last-child">
                { child.clone() }
            </div>
        }
    });

    html! {
        <div class={style}>
            <div class={"aspect-ratio-container"}>
                { first_child }
                { last_child }
            </div>
        </div>
    }
}
