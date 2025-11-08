use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::use_api;

#[function_component(LogoutPage)]
pub fn logout() -> Html {
    let api = use_api();
    {
        let api = api.clone();
        use_effect_with((), move |_| {
            let api = api.clone();
            spawn_local(async move {
                api.logout().await.unwrap();
                api.route_login();
            });
            || ()
        });
    }

    html! {}
}
