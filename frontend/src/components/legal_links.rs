use stylist::{style, yew::styled_component};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::route::Route;

#[styled_component(LegalLinks)]
pub fn legal_links() -> Html {
    let nav_style = style!(
        r#"
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 12px;
            flex-wrap: wrap;
            font-size: 0.85rem;
            color: var(--text-muted);

            & a {
                color: var(--primary);
                text-decoration: none;
                font-weight: 500;
            }

            & a:hover {
                text-decoration: underline;
            }

            & .legal-links__divider {
                color: var(--text-muted);
            }
        "#
    )
    .expect("legal nav styles");

    html! {
        <nav class={nav_style} aria-label="Rechtliche Hinweise">
            <Link<Route> to={Route::Imprint}>{ "Impressum" }</Link<Route>>
            <span aria-hidden="true" class="legal-links__divider">{ "|" }</span>
            <Link<Route> to={Route::Privacy}>{ "Datenschutz" }</Link<Route>>
            <span aria-hidden="true" class="legal-links__divider">{ "|" }</span>
            <Link<Route> to={Route::Terms}>{ "AGB" }</Link<Route>>
        </nav>
    }
}
