use super::Navable;

use stylist::yew::Global;
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

    let account_route = props
        .account_route
        .clone()
        .map(|route| route.to_nav_item().build(&navigator, location.path()))
        .unwrap_or(html! {
            <li><span class="material-symbols-outlined account">
                {"account_circle"}
            </span></li>
        });

    html! {
        <>
            <Global css={include_str!("vertical.css")} />
            if !props.fullscreen {
                <header id="vertical-layout">
                    { account_route }
                </header>
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
