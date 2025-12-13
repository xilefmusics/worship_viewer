use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    #[prop_or_default]
    pub children: ChildrenWithProps<SidebarPanel>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let current_index = use_state(|| 0);

    let controlls = props.children.iter().enumerate().map(|(index, panel) | {
        let onclick = {
            let current_index = current_index.clone();
            Callback::from(move |_| {
                current_index.set(index);
            })
        };
        html! {
            <button 
                class={classes!{"material-symbols-outlined", if *current_index == index { "selected" } else { "" }}}
                onclick={onclick}
            >
                {panel.props.icon.clone()}
            </button>
        }
    });

    let panel = props.children.iter().nth(*current_index).unwrap();

    html! {
        <div class={Style::new(include_str!("sidebar.css")).expect("Unwrapping CSS should work!")}>
            <div class="sidebar-controls">
                {for controlls}
            </div>
            {panel}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SidebarPanelProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub icon: String,
}

#[function_component(SidebarPanel)]
pub fn sidebar_panel(props: &SidebarPanelProps) -> Html {
    html! {
        <div class="sidebar-panel">
            { props.children.clone() }
        </div>
    }
}