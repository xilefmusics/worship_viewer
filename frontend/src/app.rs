use super::route::Route;
use crate::api::ApiProvider;
use fancy_yew::layouts::Navable;
use yew::prelude::*;
use yew_router::prelude::*;

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
