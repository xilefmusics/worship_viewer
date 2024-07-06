mod collections;
mod navigation_bar;
mod player;
mod routes;
mod setlist;
mod songs;
mod top_bar;

use collections::CollectionsComponent;
use player::PlayerComponent;
use routes::Route;
use setlist::SetlistComponent;
use songs::SongsComponent;
use stylist::{css, yew::Global, Style};
use yew::prelude::*;
use yew_router::prelude::*;

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <CollectionsComponent /> },
        Route::Collections => html! { <CollectionsComponent /> },
        Route::Songs => html! { <SongsComponent /> },
        Route::Setlists => html! { <SetlistComponent /> },
        Route::Player => html! {<PlayerComponent />},
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <>
            <Global css={css!("html,body{padding: 0;margin: 0;border: 0;background: #1e1e1e; overflow: hidden; overscroll-behavior: none; }")} />
            <div class={Style::new(include_str!("style.css")).expect("Unwrapping CSS should work!")}>
                <div class={"app"}>
                    <BrowserRouter>
                        <Switch<Route> render={switch} />
                    </BrowserRouter>
                </div>
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
