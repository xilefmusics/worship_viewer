use crate::api::use_api;
use gloo::console;
use gloo::file::futures::read_as_bytes;
use gloo::file::Blob as GlooBlob;
use shared::user::User;
use stylist::yew::Global;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

fn profile_picture_src(user: &User) -> Option<String> {
    user.avatar_blob_id
        .as_ref()
        .or(user.oauth_avatar_blob_id.as_ref())
        .map(|id| format!("/api/v1/blobs/{id}/data"))
}

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

    let on_avatar_file = {
        let user = user.clone();
        let api = api.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let Some(files) = input.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
            let mime = file.type_();
            let mime = mime.trim();
            if mime != "image/jpeg" && mime != "image/png" {
                console::error!("Choose a JPEG or PNG image.");
                input.set_value("");
                return;
            }
            let user = user.clone();
            let api = api.clone();
            let mime_owned = mime.to_string();
            spawn_local(async move {
                let blob = GlooBlob::from(file);
                match read_as_bytes(&blob).await {
                    Ok(bytes) => {
                        if let Ok(updated) = api
                            .upload_profile_picture(mime_owned.as_str(), &bytes)
                            .await
                        {
                            user.set(Some(updated));
                        }
                    }
                    Err(err) => console::error!(format!("Could not read file: {err:?}")),
                }
            });
            input.set_value("");
        })
    };

    let on_remove_upload = {
        let user = user.clone();
        let api = api.clone();
        Callback::from(move |_| {
            let user = user.clone();
            let api = api.clone();
            spawn_local(async move {
                if let Ok(updated) = api.delete_uploaded_profile_picture().await {
                    user.set(Some(updated));
                }
            });
        })
    };

    html! {
        <>
            <Global css={include_str!("side_menu.css")} />
            if props.open {
                <div class="side-menu-overlay" onclick={close_overlay}>
                    <div class="side-menu-panel" onclick={|e: MouseEvent| e.stop_propagation()}>
                        <div class="side-menu-header">
                            <button type="button" class="side-menu-close" onclick={close_button}>
                                <span class="material-symbols-outlined">{"close"}</span>
                            </button>
                        </div>
                        <div class="side-menu-user">
                            <div class="side-menu-avatar-wrap">
                                {if let Some(ref u) = *user {
                                    if let Some(src) = profile_picture_src(u) {
                                        html! {
                                            <img class="side-menu-avatar-img" src={src} alt="" />
                                        }
                                    } else {
                                        html! {
                                            <span class="material-symbols-outlined side-menu-avatar">
                                                {"account_circle"}
                                            </span>
                                        }
                                    }
                                } else {
                                    html! {
                                        <span class="material-symbols-outlined side-menu-avatar">
                                            {"account_circle"}
                                        </span>
                                    }
                                }}
                            </div>
                            <div class="side-menu-user-text">
                                {if let Some(user_data) = (*user).clone() {
                                    html! {
                                        <>
                                            <span class="side-menu-email">{&user_data.email}</span>
                                            <input
                                                id="side-menu-avatar-file"
                                                type="file"
                                                accept="image/jpeg,image/png"
                                                class="side-menu-avatar-file-input"
                                                onchange={on_avatar_file}
                                            />
                                            <label for="side-menu-avatar-file" class="side-menu-avatar-upload">
                                                {"Upload photo"}
                                            </label>
                                            if user_data.avatar_blob_id.is_some() {
                                                <button
                                                    type="button"
                                                    class="side-menu-avatar-remove"
                                                    onclick={on_remove_upload}
                                                >
                                                    {"Use account photo"}
                                                </button>
                                            }
                                        </>
                                    }
                                } else {
                                    html! {
                                        <span class="side-menu-email">{"Loading..."}</span>
                                    }
                                }}
                            </div>
                        </div>
                        <div class="side-menu-content">
                        </div>
                        <button type="button" class="side-menu-logout" onclick={handle_logout}>
                            {"Logout"}
                        </button>
                    </div>
                </div>
            }
        </>
    }
}
