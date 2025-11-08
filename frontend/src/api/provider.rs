use yew::prelude::*;
use yew_router::prelude::*;

use super::Api;

pub const API_BASE_URL: &str = "http://localhost:3001";

#[derive(Properties, PartialEq)]
pub struct ApiProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ApiProvider)]
pub fn api_provider(props: &ApiProviderProps) -> Html {
    let navigator = use_navigator().unwrap();
    let api = {
        let navigator = navigator.clone();
        use_memo((), move |_| Api::new(API_BASE_URL.to_string(), navigator))
    };

    html! {
        <ContextProvider<Api> context={(*api).clone()}>
            { for props.children.iter() }
        </ContextProvider<Api>>
    }
}

#[hook]
pub fn use_api() -> Api {
    use_context::<Api>().expect("Api context is missing")
}
