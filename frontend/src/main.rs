mod api;
mod app;
mod components;
mod pages;
mod route;

use app::App;

#[cfg(target_arch = "wasm32")]
fn init_pwa() {
    register_service_worker();
}

#[cfg(target_arch = "wasm32")]
fn register_service_worker() {
    use gloo_console::warn;
    use wasm_bindgen_futures::{spawn_local, JsFuture};

    spawn_local(async {
        let Some(window) = web_sys::window() else {
            return;
        };

        let Ok(protocol) = window.location().protocol() else {
            return;
        };

        if protocol != "https:" && protocol != "http:" {
            return;
        }

        let navigator = window.navigator();
        let Some(service_worker) = navigator.service_worker() else {
            return;
        };

        match service_worker.register("/service-worker.js") {
            Ok(promise) => {
                if let Err(err) = JsFuture::from(promise).await {
                    warn!(format!("service worker registration failed: {:?}", err));
                }
            }
            Err(err) => {
                warn!(format!("service worker registration error: {:?}", err));
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn init_pwa() {}

fn main() {
    init_pwa();
    yew::Renderer::<App>::new().render();
}
