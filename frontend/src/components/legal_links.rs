use stylist::{style, yew::styled_component};
use yew::prelude::*;

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
                display: inline-flex;
                flex-direction: column;
                text-align: center;
                line-height: 1.2;
                gap: 2px;
                color: var(--primary);
                text-decoration: none;
                font-weight: 500;
                white-space: normal;
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
            <a href="https://worshipviewer.com/imprint">
                { "Imprint" }
            </a>
            <span aria-hidden="true" class="legal-links__divider">{ "|" }</span>
            <a href="https://worshipviewer.com/privacy">
                { "Privacy Policy" }
            </a>
            <span aria-hidden="true" class="legal-links__divider">{ "|" }</span>
            <a href="https://worshipviewer.com/terms">
                { "Terms & Conditions" }
            </a>
        </nav>
    }
}
