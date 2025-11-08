use stylist::{style, yew::styled_component};
use yew::prelude::*;

use crate::components::LegalLinks;

#[styled_component(PrivacyPage)]
pub fn privacy_page() -> Html {
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
    .expect("privacy styles");

    html! {
        <div class={page_style}>
            <main>
                <section class="legal-card">
                    <h1>{ "Datenschutzerklärung" }</h1>
                    <p class="legal-note">{ "Diese Vorlage dient als Ausgangspunkt. Bitte ergänzen oder ersetzen Sie die Inhalte entsprechend Ihrem tatsächlichen Verarbeitungsverzeichnis und Ihren technischen sowie organisatorischen Maßnahmen." }</p>

                    <div class="legal-block">
                        <h2>{ "1. Verantwortliche Stelle" }</h2>
                        <p>
                            { "Unternehmensname GmbH" }<br />
                            { "Musterstraße 1" }<br />
                            { "12345 Musterstadt" }<br />
                            { "E-Mail: privacy@example.com" }<br />
                            { "Telefon: +49 (0) 123 456789" }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "2. Erhebung und Speicherung personenbezogener Daten" }</h2>
                        <p>{ "Wir verarbeiten personenbezogene Daten, die beim Besuch dieser Website, bei der Nutzung unseres Log-in-Bereichs oder bei der Kontaktaufnahme anfallen. Dazu zählen insbesondere:" }</p>
                        <ul>
                            <li>{ "Basisdaten wie Name, Unternehmenszugehörigkeit und Kontaktdaten" }</li>
                            <li>{ "Zugangsdaten wie E-Mail-Adresse, Login-Zeitpunkte sowie Browser-Informationen" }</li>
                            <li>{ "Kommunikationsinhalte, wenn Sie uns per E-Mail kontaktieren" }</li>
                        </ul>
                    </div>

                    <div class="legal-block">
                        <h2>{ "3. Zwecke und Rechtsgrundlagen der Verarbeitung" }</h2>
                        <p>{ "Die Verarbeitung erfolgt zur Vertragserfüllung und Durchführung vorvertraglicher Maßnahmen (Art. 6 Abs. 1 lit. b DSGVO), zur Wahrung berechtigter Interessen (Art. 6 Abs. 1 lit. f DSGVO) sowie, soweit erforderlich, auf Grundlage Ihrer Einwilligung (Art. 6 Abs. 1 lit. a DSGVO)." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "4. Weitergabe von Daten" }</h2>
                        <p>{ "Eine Übermittlung Ihrer persönlichen Daten an Dritte findet nur statt, sofern dies zur Vertragserfüllung erforderlich ist, gesetzlich vorgeschrieben ist oder Sie eingewilligt haben. Dienstleister, die wir im Rahmen einer Auftragsverarbeitung einsetzen, werden vertraglich zur Einhaltung der Datenschutzstandards verpflichtet." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "5. Speicherung und Löschung" }</h2>
                        <p>{ "Personenbezogene Daten werden nur so lange gespeichert, wie es für die jeweiligen Zwecke erforderlich ist oder gesetzliche Aufbewahrungsfristen bestehen. Danach werden die Daten gelöscht bzw. anonymisiert." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "6. Ihre Rechte" }</h2>
                        <p>{ "Sie haben das Recht auf Auskunft, Berichtigung, Löschung, Einschränkung der Verarbeitung sowie Datenübertragbarkeit. Außerdem steht Ihnen ein Widerspruchsrecht gegen die Verarbeitung personenbezogener Daten zu, die wir auf Basis berechtigter Interessen verarbeiten." }</p>
                        <p>{ "Wenden Sie sich hierzu bitte an die oben genannte verantwortliche Stelle. Ihnen steht zudem ein Beschwerderecht bei einer Datenschutzaufsichtsbehörde zu." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "7. Datensicherheit" }</h2>
                        <p>{ "Wir setzen technische und organisatorische Sicherheitsmaßnahmen ein, um Ihre Daten gegen Manipulation, Verlust, Zerstörung oder unbefugten Zugriff zu schützen. Unsere Maßnahmen werden entsprechend der technologischen Entwicklung fortlaufend verbessert." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "8. Aktualität und Änderung dieser Datenschutzerklärung" }</h2>
                        <p>{ "Wir behalten uns vor, diese Datenschutzerklärung zu aktualisieren, damit sie stets den aktuellen rechtlichen Anforderungen entspricht oder um Änderungen unserer Leistungen in der Datenschutzerklärung umzusetzen." }</p>
                    </div>

                    <LegalLinks />
                </section>
            </main>
        </div>
    }
}
