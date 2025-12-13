use super::OutlineData;
use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct OutlineProps {
    #[prop_or_default]
    pub data: Vec<OutlineData>,
    pub set_current: Callback<(usize, usize)>,
    pub current_text: usize,
    pub current_outline: usize,
}

#[function_component(Outline)]
pub fn outline(props: &OutlineProps) -> Html {
    let items = props
        .data
        .iter()
        .map(|item| {
            let len = if props.current_outline != usize::MAX {
                if props.current_outline >= item.outline_idx && props.current_outline < item.outline_idx + item.len {
                    item.len
                } else {
                    1
                }
            } else {
                if props.current_text >= item.text_idx && props.current_text < item.text_idx + item.len {
                    item.len
                } else {
                    1
                }
            };

            (0..len).map(|offset| {
            let onclick = {
                let set_current = props.set_current.clone();
                let text_idx = item.text_idx;
                let outline_idx = item.outline_idx;
                Callback::from(move |_| {
                    set_current.emit((text_idx+offset, outline_idx+offset));
                })
            };

            let selected = if props.current_outline != usize::MAX {
                item.outline_idx+offset == props.current_outline
            } else {
                item.text_idx+offset == props.current_text
            };

            html! {
                <li
                    {onclick}
                    class={classes!{
                        if selected { "selected" } else { "" },
                        if !item.has_text { "no-text" } else { "" },
                        if offset != 0 { "not-first" } else { "" },
                    }}
                >
                    {if offset == 0 { item.title.clone() } else { format!("{} ({})", item.title, offset+1)}}
                </li>
            }}).collect::<Html>()
        })
        .collect::<Html>();

    html! {
        <div class={Style::new(include_str!("outline.css")).expect("Unwrapping CSS should work!")}>
            <ul>
                {items}
            </ul>
        </div>
    }
}
