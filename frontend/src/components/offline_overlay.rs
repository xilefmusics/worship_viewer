use gloo::events::EventListener;
use stylist::Style;
use yew::prelude::*;

#[function_component(OfflineOverlay)]
pub fn offline_overlay() -> Html {
    let stylesheet = use_memo((), |_| {
        Style::new(include_str!("offline_overlay.css")).expect("valid CSS")
    });

    let is_online = use_state(|| {
        web_sys::window()
            .map(|window| window.navigator().on_line())
            .unwrap_or(true)
    });

    {
        let is_online = is_online.clone();
        use_effect_with((), move |_| -> Box<dyn FnOnce()> {
            if let Some(window) = web_sys::window() {
                let offline_state = is_online.clone();
                let offline_listener = EventListener::new(&window, "offline", move |_| {
                    offline_state.set(false);
                });

                let online_state = is_online.clone();
                let online_listener = EventListener::new(&window, "online", move |_| {
                    online_state.set(true);
                });

                Box::new(move || {
                    drop(offline_listener);
                    drop(online_listener);
                })
            } else {
                Box::new(|| ())
            }
        });
    }

    if *is_online {
        return Html::default();
    }

    let handle_retry = Callback::from(|_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    });

    html! {
        <div class={classes!(stylesheet.get_class_name().to_string(), "offline-overlay")}>
            <div class="offline-card" role="alert" aria-live="assertive">
                <div class="offline-icon" aria-hidden="true">{ "!" }</div>
                <h1>{ "You're offline" }</h1>
                <p>{ "Previously loaded screens remain available. We'll sync once you're back online." }</p>
                <div class="offline-actions">
                    <button class="offline-primary" onclick={handle_retry}>{ "Try again" }</button>
                    <span class="offline-caption">{ "Stay on this page to keep working." }</span>
                </div>
            </div>
        </div>
    }
}
