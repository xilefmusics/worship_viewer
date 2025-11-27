use super::{Navable, SideMenu};

use stylist::yew::Global;
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props<R: Routable + Navable> {
    pub children: Html,
    pub nav_routes: Vec<R>,
    #[prop_or_default]
    pub account_route: Option<R>,
    #[prop_or_default]
    pub fullscreen: bool,
}

#[function_component(VerticalLayout)]
pub fn vertical_layout<R: Routable + Navable>(props: &Props<R>) -> Html {
    let navigator = use_navigator().unwrap();
    let location = use_location().unwrap();
    let menu_open = use_state(|| false);

    let nav_items = props
        .nav_routes
        .iter()
        .map(|route| {
            route
                .clone()
                .to_nav_item()
                .build(&navigator, location.path())
        })
        .collect::<Html>();

    let toggle_menu = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            menu_open.set(!*menu_open);
        })
    };

    let close_menu = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            menu_open.set(false);
        })
    };

    let handle_logout = Callback::from(move |_| {
        if let Some(window) = window() {
            let location = window.location();
            let _ = location.set_href("/logout");
        }
    });

    let account_route_html = props
        .account_route
        .clone()
        .map(|route| route.to_nav_item().build(&navigator, location.path()))
        .unwrap_or(html! {
            <li onclick={toggle_menu.clone()} style="cursor: pointer;">
                <span class="material-symbols-outlined account">
                    {"account_circle"}
                </span>
            </li>
        });

    html! {
        <>
            <Global css={include_str!("vertical.css")} />
            if !props.fullscreen {
                <header id="vertical-layout">
                    <div
                        onclick={toggle_menu.clone()}
                        style="cursor: pointer; display: contents;"
                        onmousedown={|e: MouseEvent| e.stop_propagation()}
                    >
                        { account_route_html }
                    </div>
                </header>
                <SideMenu
                    open={*menu_open}
                    on_close={close_menu}
                    on_logout={handle_logout}
                />
            }
            <main id="vertical-layout">
                {
                    props.children.clone()
                }
            </main>
            if !props.fullscreen {
                <footer id="vertical-layout">
                    <nav id="vertical-layout">
                        <ul>
                        {
                            nav_items
                        }
                        </ul>
                    </nav>
                </footer>
            }
        </>
    }
}
