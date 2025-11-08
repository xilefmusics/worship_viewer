use super::route::Route;
use fancy_yew::layouts::Navable;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::api::ApiProvider;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <ApiProvider>
                <Switch<Route> render={Route::render} />
            </ApiProvider>
        </BrowserRouter>
    }
}
