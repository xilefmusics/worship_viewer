use crate::api::use_api;
use shared::user::User;
use stylist::yew::Global;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub open: bool,
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
}

#[function_component(SideMenu)]
pub fn side_menu(props: &Props) -> Html {
    let user = use_state(|| None::<User>);
    let api = use_api();

    // Fetch user data when menu opens
    {
        let user = user.clone();
        let api = api.clone();
        let open = props.open;
        use_effect_with(open, move |open| {
            if *open {
                let user = user.clone();
                let api = api.clone();
                spawn_local(async move {
                    if let Ok(fetched_user) = api.get_users_me().await {
                        user.set(Some(fetched_user));
                    }
                });
            }
            || ()
        });
    }

    let close_overlay = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    let close_button = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    let handle_logout = {
        let on_close = props.on_close.clone();
        let on_logout = props.on_logout.clone();
        Callback::from(move |_| {
            on_close.emit(());
            on_logout.emit(());
        })
    };

    html! {
        <>
            <Global css={include_str!("side_menu.css")} />
            if props.open {
                <div class="side-menu-overlay" onclick={close_overlay}>
                    <div class="side-menu-panel" onclick={|e: MouseEvent| e.stop_propagation()}>
                        <div class="side-menu-header">
                            <button class="side-menu-close" onclick={close_button}>
                                <span class="material-symbols-outlined">{"close"}</span>
                            </button>
                        </div>
                        <div class="side-menu-user">
                            <span class="material-symbols-outlined side-menu-avatar">
                                {"account_circle"}
                            </span>
                            {if let Some(user_data) = (*user).clone() {
                                html! {
                                    <span class="side-menu-email">{&user_data.email}</span>
                                }
                            } else {
                                html! {
                                    <span class="side-menu-email">{"Loading..."}</span>
                                }
                            }}
                        </div>
                        <div class="side-menu-content">
                        </div>
                        <button class="side-menu-logout" onclick={handle_logout}>
                            {"Logout"}
                        </button>
                    </div>
                </div>
            }
        </>
    }
}
