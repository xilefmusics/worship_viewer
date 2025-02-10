use super::pages::{CollectionsPage, EditorPage, IndexPage, PlayerPage, SetlistsPage, SongsPage};
use fancy_yew::layouts::{NavItemBuilder, Navable, VerticalLayout as Layout};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,
    #[at("/collections")]
    Collections,
    #[at("/songs")]
    Songs,
    #[at("/setlists")]
    Setlists,
    #[at("/player")]
    Player,
    #[at("/editor")]
    Editor,
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Navable for Route {
    fn route_items() -> Vec<Self> {
        vec![Route::Collections, Route::Songs, Route::Setlists]
    }

    fn to_nav_item(self) -> NavItemBuilder<'static> {
        match self {
            Route::Index => NavItemBuilder::new()
                .path("/home")
                .callback(Callback::from(|navigator: Navigator| {
                    navigator.push(&Route::Index)
                }))
                .index(),
            Route::Collections => NavItemBuilder::new()
                .path("/collections")
                .icon("menu_book")
                .callback(Callback::from(|navigator: Navigator| {
                    navigator.push(&Route::Collections)
                })),
            Route::Songs => NavItemBuilder::new()
                .path("/songs")
                .icon("library_music")
                .callback(Callback::from(|navigator: Navigator| {
                    navigator.push(&Route::Songs)
                })),
            Route::Setlists => NavItemBuilder::new()
                .path("/setlists")
                .icon("receipt_long")
                .callback(Callback::from(|navigator: Navigator| {
                    navigator.push(&Route::Setlists)
                })),
            _ => NavItemBuilder::new(),
        }
    }

    fn render(route: Route) -> Html {
        html! {
            <Layout<Route>
                nav_routes={Route::route_items()}
                fullscreen={match route {
                    Route::Player => true,
                    Route::Editor => true,
                    _ => false,
                }}
            >{
                match route {
                    Route::Index => html! { <IndexPage /> },
                    Route::Collections => html! { <CollectionsPage /> },
                    Route::Songs => html! { <SongsPage /> },
                    Route::Setlists => html! { <SetlistsPage /> },
                    Route::Player => html! { <PlayerPage /> },
                    Route::Editor => html! { <EditorPage /> },
                    Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
        }}
            </Layout<Route>>
        }
    }
}
