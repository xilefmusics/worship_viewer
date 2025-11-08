mod app;
mod components;
mod pages;
mod route;
mod api;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
