use crate::route::Route;

use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(IndexPage)]
pub fn index_page() -> Html {
    let navigator = use_navigator().unwrap();
    navigator.push(&Route::Collections);
    html! {}
}