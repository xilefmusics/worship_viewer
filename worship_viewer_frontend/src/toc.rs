use stylist::Style;
use worship_viewer_shared::types::TocItem;
use yew::prelude::*;

#[derive(PartialEq, Clone)]
enum FilterSort {
    Real,
    Alphabetical,
    Liked,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub list: Vec<TocItem>,
    pub select: Callback<usize>,
}

#[function_component]
pub fn TableOfContentsComponent(props: &Props) -> Html {
    let select = props.select.clone();
    let list = props.list.clone();

    let filter_sort = use_state(|| FilterSort::Real);

    let oncklick_real = {
        let filterSort = filter_sort.clone();
        move |_: MouseEvent| {
            filterSort.set(FilterSort::Real);
        }
    };
    let oncklick_alphabetical = {
        let filterSort = filter_sort.clone();
        move |_: MouseEvent| {
            filterSort.set(FilterSort::Alphabetical);
        }
    };
    let oncklick_liked = {
        let filterSort = filter_sort.clone();
        move |_: MouseEvent| {
            filterSort.set(FilterSort::Liked);
        }
    };

    let list = match *filter_sort {
        FilterSort::Real => list,
        FilterSort::Alphabetical => {
            let mut result = list;
            result.sort_by_key(|item| item.title.clone());
            result
        }
        FilterSort::Liked => vec![],
    };

    let list = list
        .iter()
        .map(|item| {
            let onclick = {
                let select = select.clone();
                let idx = item.idx;
                move |_: MouseEvent| select.emit(idx)
            };
            if *filter_sort == FilterSort::Real {
                html! {
                    <li onclick={onclick}>{format!("{}. {}", &item.nr, &item.title)}</li>
                }
            } else {
                html! {
                    <li onclick={onclick}>{&item.title}</li>
                }
            }
        })
        .collect::<Html>();

    html! {
        <div class={Style::new(include_str!("toc.css")).expect("Unwrapping CSS should work!")}>
            <div>
                <span
                    class={if *filter_sort == FilterSort::Real {"material-symbols-outlined filter-sort selected"} else {"material-symbols-outlined filter-sort"}}
                    onclick={oncklick_real}
                >{"pin"}</span>
                <span
                    class={if *filter_sort == FilterSort::Alphabetical {"material-symbols-outlined filter-sort selected"} else {"material-symbols-outlined filter-sort"}}
                    onclick={oncklick_alphabetical}
                >{"sort_by_alpha"}</span>
                <span
                    class={if *filter_sort == FilterSort::Liked {"material-symbols-outlined filter-sort selected"} else {"material-symbols-outlined filter-sort"}}
                    onclick={oncklick_liked}
                >{"favorite"}</span>
            </div>
            <ul>{list}</ul>
        </div>
    }
}
