use stylist::{style, yew::styled_component};
use yew::prelude::*;

use crate::components::LegalLinks;

#[styled_component(TermsPage)]
pub fn terms_page() -> Html {
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
    .expect("terms styles");

    html! {
        <div class={page_style}>
            <main>
                <section class="legal-card">
                    <h1>{ "Allgemeine Geschäftsbedingungen (AGB)" }</h1>
                    <p class="legal-note">{ "Diese AGB-Vorlage stellt lediglich einen Vorschlag dar. Bitte prüfen Sie die Inhalte mit Ihrer Rechtsberatung, bevor Sie sie produktiv einsetzen." }</p>

                    <div class="legal-block">
                        <h2>{ "1. Geltungsbereich" }</h2>
                        <p>{ "Diese Allgemeinen Geschäftsbedingungen gelten für alle Verträge zwischen der Unternehmensname GmbH (nachfolgend „Anbieter“) und ihren Kundinnen und Kunden (nachfolgend „Kunden“) über die Nutzung der angebotenen Software- und Beratungsleistungen." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "2. Vertragsgegenstand" }</h2>
                        <p>{ "Der Anbieter stellt den Kunden eine digitale Plattform zur Authentifizierung sowie ergänzende Dienstleistungen zur Verfügung. Der genaue Leistungsumfang ergibt sich aus dem jeweils geschlossenen Einzelvertrag bzw. dem aktuellen Leistungsbeschrieb." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "3. Vertragsschluss" }</h2>
                        <p>{ "Ein Vertrag kommt durch schriftliche Auftragsbestätigung oder durch Nutzung der Dienstleistung nach Registrierung zustande. Der Anbieter behält sich das Recht vor, Anfragen ohne Angabe von Gründen abzulehnen." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "4. Preise und Zahlungsbedingungen" }</h2>
                        <p>{ "Soweit nicht anders vereinbart, gelten die auf der Website ausgewiesenen Preise zuzüglich gesetzlicher Umsatzsteuer. Rechnungen sind innerhalb von 14 Tagen ohne Abzug zur Zahlung fällig." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "5. Pflichten der Kunden" }</h2>
                        <p>{ "Kunden sind verpflichtet, Zugangsdaten vertraulich zu behandeln und unbefugten Dritten keinen Zugriff zu ermöglichen. Änderungen der Kontakt- oder Rechnungsdaten sind unverzüglich mitzuteilen." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "6. Haftung" }</h2>
                        <p>{ "Der Anbieter haftet für Vorsatz und grobe Fahrlässigkeit unbeschränkt. Bei leicht fahrlässiger Verletzung einer wesentlichen Vertragspflicht ist die Haftung auf den vorhersehbaren, vertragstypischen Schaden begrenzt. Die Haftung nach dem Produkthaftungsgesetz bleibt unberührt." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "7. Laufzeit und Kündigung" }</h2>
                        <p>{ "Die Vertragslaufzeit ergibt sich aus dem jeweiligen Einzelvertrag. Beide Parteien können den Vertrag aus wichtigem Grund ohne Einhaltung einer Frist kündigen. Das Recht zur ordentlichen Kündigung bleibt unberührt." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "8. Schlussbestimmungen" }</h2>
                        <p>{ "Es gilt das Recht der Bundesrepublik Deutschland unter Ausschluss des UN-Kaufrechts. Erfüllungsort und Gerichtsstand ist, soweit zulässig, der Sitz des Anbieters. Sollten einzelne Bestimmungen unwirksam sein, bleibt die Wirksamkeit der übrigen Regelungen unberührt." }</p>
                    </div>

                    <LegalLinks />
                </section>
            </main>
        </div>
    }
}
