use stylist::Style;
use yew::prelude::*;

#[derive(PartialEq, Clone)]
pub struct TocItem {
    pub idx: usize,
    pub text: String,
}

#[derive(Properties, PartialEq)]
pub struct TocProps {
    #[prop_or_default]
    pub list: Vec<TocItem>,
    pub select: Callback<usize>,
    pub current_idx: usize,
}

#[function_component(Toc)]
pub fn toc(props: &TocProps) -> Html {
    html! {
        <div class={Style::new(include_str!("toc.css")).expect("Unwrapping CSS should work!")}>
            <ul>
                {props.list.iter().map(|item| {
                    let onclick = {
                        let select = props.select.clone();
                        let idx = item.idx;
                        move |_: MouseEvent| select.emit(idx)
                    };
                    html! {
                        <li 
                            {onclick}
                            class={if item.idx == props.current_idx { "selected" } else { "" }}
                        >
                            {item.text.clone()}
                        </li>
                    }
                }).collect::<Html>()}
            </ul>
        </div>
    }
}
