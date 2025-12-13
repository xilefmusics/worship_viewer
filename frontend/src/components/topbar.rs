use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TopbarProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Topbar)]
pub fn topbar(props: &TopbarProps) -> Html {
    html! {
        <div class={Style::new(include_str!("topbar.css")).expect("Unwrapping CSS should work!")}>
            { props.children.clone() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TopbarButtonProps {
    #[prop_or_default]
    pub icon: String,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(TopbarButton)]
pub fn topbar_button(props: &TopbarButtonProps) -> Html {
    html! {
        <button
            class="material-symbols-outlined button"
            onclick={props.onclick.clone()}
        >{&props.icon}</button>
    }
}

#[function_component(TopbarSpacer)]
pub fn topbar_spacer() -> Html {
    html! {
        <div class="seperator"></div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TopbarSelectProps {
    #[prop_or_default]
    pub children: ChildrenWithProps<TopbarSelectOption>,
}

#[function_component(TopbarSelect)]
pub fn topbar_select(props: &TopbarSelectProps) -> Html {
    let active = use_state(|| false);
    if !*active {
        let selected = props
            .children
            .iter()
            .filter(|child| child.props.selected)
            .map(|child| {
                html! {
                    <TopbarSelectOption
                        icon={child.props.icon.clone()}
                        text={child.props.text.clone()}
                        selected={child.props.selected}
                        onclick={let active = active.clone(); move |_| { active.set(true)}}
                     />
                }
            })
            .collect::<Html>();
        html! {
            <ul>
                {selected}
            </ul>
        }
    } else {
        let children = props
            .children
            .iter()
            .map(|child| {
                html! {child.clone()}
            })
            .collect::<Html>();

        html! {
            <ul 
                class="active"
                onclick={let active = active.clone(); move |_| { active.set(false)}}
            >
                {children}
            </ul>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct TopbarSelectOptionProps {
    #[prop_or_default]
    pub icon: String,
    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(TopbarSelectOption)]
pub fn topbar_select_option(props: &TopbarSelectOptionProps) -> Html {
    html! {
        <li class={if props.selected { "selected" } else { "" }}
            onclick={props.onclick.clone()}>
            <span class="material-symbols-outlined">{&props.icon}</span>
            <span>{&props.text}</span>
        </li>
    }
}
