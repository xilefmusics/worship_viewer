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

                    <div class="legal-block">
                        <h2>{ "1. Geltungsbereich" }</h2>
                        <p>
                            { "Diese Allgemeinen Geschäftsbedingungen gelten für die Nutzung der App „WorshipViewer“ (nachfolgend „App“) durch registrierte und nicht registrierte Nutzerinnen und Nutzer (nachfolgend „Nutzer“). Abweichende Bedingungen der Nutzer finden keine Anwendung." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "2. Anbieter" }</h2>
                        <p>
                            { "Anbieter der App ist:" }<br />
                            { "Felix Rollbühler" }<br />
                            { "Münklinger Str. 2" }<br />
                            { "75378 Bad Liebenzell" }<br />
                            { "E-Mail: info@worshipviewer.com" }
                        </p>
                        <p class="legal-note">
                            { "Die App wird privat betrieben und dient nicht kommerziellen Zwecken." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "3. Leistungsbeschreibung" }</h2>
                        <p>
                            { "Der Anbieter stellt die App in der jeweils aktuellen Version zur Verfügung. Funktionsumfang und Verfügbarkeit können variieren. Ein Anspruch auf bestimmte Funktionen, eine bestimmte Verfügbarkeit oder Support besteht nicht." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "4. Registrierung und Konto" }</h2>
                        <p>
                            { "Für bestimmte Funktionen ist eine Registrierung erforderlich. Die bei der Registrierung abgefragten Daten sind wahrheitsgemäß anzugeben und aktuell zu halten. Zugangsdaten sind geheim zu halten und vor dem Zugriff Dritter zu schützen. Der Anbieter kann Konten sperren oder löschen, wenn ein Missbrauch vorliegt oder berechtigte Gründe dafür bestehen." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "5. Nutzungsregeln / Missbrauch" }</h2>
                        <ul>
                            <li>{ "Die App darf nur im Rahmen der geltenden Gesetze und dieser AGB genutzt werden." }</li>
                            <li>{ "Untersagt sind insbesondere: Sicherheitsrelevante Angriffe, automatisierte Massenanfragen, Umgehung technischer Schutzmaßnahmen, Veröffentlichung rechtswidriger Inhalte." }</li>
                            <li>{ "Der Anbieter kann zur Sicherstellung des Betriebs angemessene Nutzungsbeschränkungen (z. B. Rate-Limits) einführen." }</li>
                        </ul>
                    </div>

                    <div class="legal-block">
                        <h2>{ "6. Entgeltlichkeit" }</h2>
                        <p>
                            { "Die Nutzung der App ist derzeit kostenlos. Sollte der Anbieter zukünftig kostenpflichtige Funktionen einführen, werden Nutzer hierüber vorab informiert. In diesem Fall gelten die zum Zeitpunkt der Einführung mitgeteilten Preise, Zahlungsbedingungen und – sofern einschlägig – Verbraucherinformationen (inkl. Widerrufsbelehrung)." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "7. Verfügbarkeit, Wartung und Änderungen" }</h2>
                        <p>
                            { "Der Anbieter bemüht sich um einen störungsfreien Betrieb der App, kann jedoch Zeiten eingeschränkter Verfügbarkeit (z. B. Wartung, Updates, höhere Gewalt) nicht ausschließen. Der Anbieter ist berechtigt, Leistungen anzupassen, zu erweitern oder einzustellen, sofern berechtigte Interessen der Nutzer angemessen berücksichtigt werden." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "8. Haftung" }</h2>
                        <p>
                            { "Der Anbieter stellt die App unentgeltlich und ohne Zusicherung bestimmter Funktionen oder Verfügbarkeiten bereit. Eine Haftung für materielle oder immaterielle Schäden, die durch die Nutzung oder Nichtnutzung der App entstehen, ist ausgeschlossen, soweit kein vorsätzliches oder grob fahrlässiges Verhalten des Anbieters vorliegt. Die Haftung bei Verletzung von Leben, Körper oder Gesundheit bleibt unberührt." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "9. Rechte an Inhalten und Software" }</h2>
                        <p>
                            { "Sämtliche Rechte an der App, insbesondere Urheber- und Schutzrechte, verbleiben beim Anbieter bzw. den jeweiligen Rechteinhabern. Nutzern wird ein einfaches, nicht übertragbares Recht eingeräumt, die App im Rahmen dieser AGB zu nutzen. Eigene Inhalte der Nutzer dürfen nur eingestellt werden, wenn sie frei von Rechten Dritter sind bzw. notwendige Rechte vorliegen." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "10. Laufzeit und Beendigung" }</h2>
                        <p>
                            { "Diese AGB gelten auf unbestimmte Zeit. Nutzer können die Nutzung jederzeit beenden und ihr Konto löschen. Der Anbieter kann Nutzerkonten aus wichtigem Grund sperren oder kündigen, insbesondere bei Verstößen gegen diese AGB oder missbräuchlicher Nutzung." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "11. Änderungen der AGB" }</h2>
                        <p>
                            { "Der Anbieter kann diese AGB mit Wirkung für die Zukunft ändern. Über wesentliche Änderungen werden Nutzer in geeigneter Form informiert. Widersprechen Nutzer der Änderung nicht innerhalb einer angemessenen Frist oder nutzen die App nach Wirksamwerden weiter, gelten die Änderungen als angenommen. Hierauf wird bei der Information gesondert hingewiesen." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "12. Datenschutz" }</h2>
                        <p>
                            { "Informationen zur Verarbeitung personenbezogener Daten finden Sie in der Datenschutzerklärung." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "13. Schlussbestimmungen" }</h2>
                        <p>
                            { "Es gilt das Recht der Bundesrepublik Deutschland unter Ausschluss des UN-Kaufrechts. Ist der Nutzer Verbraucher mit Wohnsitz in der EU, bleiben zwingende Verbraucherschutzvorschriften seines Aufenthaltsstaats unberührt. Gerichtsstand ist – soweit zulässig – der Sitz des Anbieters. Sollten einzelne Bestimmungen unwirksam sein, bleibt die Wirksamkeit der übrigen Regelungen unberührt." }
                        </p>
                    </div>

                    <p class="legal-note">{ "Stand: 10. November 2025" }</p>

                    <LegalLinks />
                </section>
            </main>
        </div>
    }
}
