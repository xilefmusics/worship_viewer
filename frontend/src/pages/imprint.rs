use stylist::{style, yew::styled_component};
use yew::prelude::*;

use crate::components::LegalLinks;

#[styled_component(ImprintPage)]
pub fn imprint_page() -> Html {
    let page_style = style!(
        r#"
            width: 100%;
            min-height: 100vh;
            padding: 24px;
            margin: 0;
            background: var(--bg-dark);
            color: var(--text);
            font-family: "Inter", "SF Pro Display", "Segoe UI", sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            box-sizing: border-box;

            & * {
                box-sizing: inherit;
            }

            & main {
                width: min(860px, 100%);
            }

            & .legal-card {
                background: var(--bg);
                border-radius: 24px;
                padding: 32px;
                box-shadow: var(--shadow-s);
                display: flex;
                flex-direction: column;
                gap: 18px;
            }

            & .legal-note {
                font-size: 0.85rem;
                color: var(--text-muted);
            }

            & .legal-block {
                display: flex;
                flex-direction: column;
                gap: 6px;
            }

            & .legal-block ul {
                margin: 0;
                padding-left: 18px;
                display: flex;
                flex-direction: column;
                gap: 4px;
            }

            & a {
                color: var(--primary);
                text-decoration: none;
                font-weight: 500;
            }

            & a:hover {
                text-decoration: underline;
            }

            @media (max-width: 480px) {
                & .legal-card {
                    padding: 24px;
                }
            }
        "#
    )
    .expect("imprint styles");

    html! {
        <div class={page_style}>
            <main>
                <section class="legal-card">
                    <h1>{ "Impressum" }</h1>
                    <p class="legal-note">{ "Bitte ersetzen Sie alle Platzhalter durch die tatsächlichen Angaben Ihres Unternehmens, um den Anforderungen der §§ 5 TMG und 55 RStV zu entsprechen." }</p>

                    <div class="legal-block">
                        <h2>{ "Angaben gemäß § 5 TMG" }</h2>
                        <p>
                            { "Unternehmensname GmbH" }<br />
                            { "Musterstraße 1" }<br />
                            { "12345 Musterstadt" }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Vertreten durch" }</h2>
                        <p>{ "Geschäftsführer: Max Mustermann" }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Kontakt" }</h2>
                        <p>
                            { "Telefon: +49 (0) 123 456789" }<br />
                            { "E-Mail: legal@example.com" }<br />
                            { "Fax: +49 (0) 123 456788" }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Registereintrag" }</h2>
                        <p>
                            { "Eintragung im Handelsregister." }<br />
                            { "Registergericht: Amtsgericht Musterstadt" }<br />
                            { "Registernummer: HRB 123456" }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Umsatzsteuer-ID" }</h2>
                        <p>{ "Umsatzsteuer-Identifikationsnummer gemäß § 27 a Umsatzsteuergesetz: DE123456789" }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Verantwortlich für den Inhalt nach § 55 Abs. 2 RStV" }</h2>
                        <p>
                            { "Max Mustermann" }<br />
                            { "Musterstraße 1" }<br />
                            { "12345 Musterstadt" }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "EU-Streitschlichtung" }</h2>
                        <p>
                            { "Die Europäische Kommission stellt eine Plattform zur Online-Streitbeilegung (OS) bereit: " }
                            <a href="https://ec.europa.eu/consumers/odr" target="_blank" rel="noreferrer">{ "https://ec.europa.eu/consumers/odr" }</a>
                            { ". Unsere E-Mail-Adresse finden Sie oben im Impressum." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "Verbraucherstreitbeilegung/Universalschlichtungsstelle" }</h2>
                        <p>{ "Wir sind nicht bereit oder verpflichtet, an Streitbeilegungsverfahren vor einer Verbraucherschlichtungsstelle teilzunehmen." }</p>
                    </div>

                    <LegalLinks />
                </section>
            </main>
        </div>
    }
}
