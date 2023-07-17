use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/collections")]
    Collections,
    #[at("/songs")]
    Songs,
    #[at("/setlists")]
    Setlists,
    #[at("/player/:id")]
    Player { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}
