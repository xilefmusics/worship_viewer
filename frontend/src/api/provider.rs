use yew::prelude::*;
use yew_router::prelude::*;

use super::Api;

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
        use_memo((), move |_| {
            let base_url = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_default();
            Api::new(navigator, base_url)
        })
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
