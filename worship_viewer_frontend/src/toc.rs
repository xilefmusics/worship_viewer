use stylist::Style;
use worship_viewer_shared::types::TocItem;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub list: Vec<TocItem>,
    pub select: Callback<usize>,
}

#[function_component]
pub fn TableOfContentsComponent(props: &Props) -> Html {
    let select = props.select.clone();
    let list = props
        .list
        .iter()
        .map(|item| {
            let onclick = {
                let select = select.clone();
                let idx = item.idx;
                move |_: MouseEvent| select.emit(idx)
            };
            html! {
                <li onclick={onclick}>{&item.title}</li>
            }
        })
        .collect::<Html>();
    html! {
        <div class={Style::new(include_str!("toc.css")).expect("Unwrapping CSS should work!")}>
            <ul>{list}</ul>
        </div>
    }
}
