use super::route::Route;
use crate::{api::ApiProvider, components::OfflineOverlay};
use fancy_yew::layouts::Navable;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <ApiProvider>
                <OfflineOverlay />
                <Switch<Route> render={Route::render} />
            </ApiProvider>
        </BrowserRouter>
    }
}
