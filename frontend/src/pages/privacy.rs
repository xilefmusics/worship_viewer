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

                    <div class="legal-block">
                        <h2>{ "1. Verantwortliche Stelle" }</h2>
                        <p>
                            { "Felix Rollbühler" }<br />
                            { "Münklinger Str. 2" }<br />
                            { "75378 Bad Liebenzell" }<br />
                            { "E-Mail: info@worshipviewer.com" }
                        </p>
                        <p class="legal-note">
                            { "Hinweis: Diese App wird privat betrieben und dient nicht kommerziellen Zwecken. Es besteht keine gesetzliche Verpflichtung zur Benennung eines Datenschutzbeauftragten." }
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "2. Erhebung und Speicherung personenbezogener Daten" }</h2>
                        <p>{ "Beim Besuch dieser Website bzw. bei Nutzung unserer App werden personenbezogene Daten nur im technisch notwendigen Umfang erhoben." }</p>

                        <p><strong>{ "a) Server-Logfiles (Hosting über Google Cloud Run)" }</strong></p>
                        <p>
                            { "Wir nutzen Google Cloud Run (Google Ireland Limited, Gordon House, Barrow Street, Dublin 4, Irland) als Hosting-Dienst. Beim Aufruf unserer App/Website werden automatisch Server-Logfiles erhoben und gespeichert (IP-Adresse, Datum und Uhrzeit des Zugriffs, Browsertyp/-version bzw. App-Version, Betriebssystem, Referrer-URL – falls übermittelt). Diese Daten dienen ausschließlich der Sicherstellung eines störungsfreien Betriebs, der IT-Sicherheit sowie der Fehleranalyse." }
                        </p>
                        <p>{ "Verarbeitungsort: Die Verarbeitung erfolgt grundsätzlich in Rechenzentren innerhalb der Europäischen Union. Eine Datenübermittlung in Drittländer findet nur statt, sofern die Voraussetzungen der Art. 44 ff. DSGVO (insb. geeignete Garantien) erfüllt sind." }</p>
                        <p>{ "Löschung der Logdaten: spätestens nach 30 Tagen." }</p>

                        <p><strong>{ "b) Nutzerkonto / Login-Bereich" }</strong></p>
                        <p>{ "Wenn Sie sich registrieren oder anmelden, speichern wir die folgenden Daten:" }</p>
                        <ul>
                            <li>{ "E-Mail-Adresse" }</li>
                            <li>{ "Zeitpunkt des letzten Logins" }</li>
                            <li>{ "Anzahl der getätigten Anfragen" }</li>
                        </ul>
                        <p>{ "Diese Daten benötigen wir, um den Zugang bereitzustellen, Missbrauch zu verhindern und einfache Nutzungsstatistiken zu führen. Eine Weitergabe an Dritte erfolgt nicht." }</p>

                        <p><strong>{ "c) Kommunikation per E-Mail" }</strong></p>
                        <p>{ "Wenn Sie uns per E-Mail kontaktieren, verarbeiten wir die von Ihnen übermittelten personenbezogenen Daten (z. B. Name, E-Mail-Adresse, Inhalte der Nachricht) zur Bearbeitung Ihres Anliegens. Nach Abschluss der Kommunikation werden diese Daten gelöscht, sofern keine gesetzlichen Aufbewahrungspflichten bestehen." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "3. Zwecke und Rechtsgrundlagen der Verarbeitung" }</h2>
                        <p>{ "Die Verarbeitung erfolgt zu folgenden Zwecken und auf folgenden Rechtsgrundlagen:" }</p>
                        <ul>
                            <li>{ "Bereitstellung, Betrieb, Stabilität und Sicherheit der App/Website (Art. 6 Abs. 1 lit. f DSGVO – berechtigtes Interesse: Betrieb, IT-Sicherheit, Fehleranalyse)" }</li>
                            <li>{ "Verwaltung von Nutzerkonten (Art. 6 Abs. 1 lit. b DSGVO – Erfüllung der Nutzungsbeziehung)" }</li>
                            <li>{ "Beantwortung von Anfragen/Support (Art. 6 Abs. 1 lit. f DSGVO – berechtigtes Interesse an effizienter Kommunikation)" }</li>
                            <li>{ "Soweit im Einzelfall erforderlich: Verarbeitung auf Grundlage Ihrer Einwilligung (Art. 6 Abs. 1 lit. a DSGVO); Einwilligungen können jederzeit mit Wirkung für die Zukunft widerrufen werden." }</li>
                        </ul>
                    </div>

                    <div class="legal-block">
                        <h2>{ "4. Weitergabe von Daten" }</h2>
                        <p>
                            { "Eine Übermittlung Ihrer personenbezogenen Daten an Dritte erfolgt nur, wenn dies gesetzlich erlaubt ist. Für das Hosting setzen wir Google Cloud Run (Google Ireland Limited) als Auftragsverarbeiter nach Art. 28 DSGVO ein; Google kann zur Erbringung der Leistungen verbundene Unternehmen/Subunternehmer einsetzen." }
                        </p>
                        <p>
                            { "Mit Google wurde ein Vertrag zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO abgeschlossen. Details: " }
                            <a href="https://cloud.google.com/terms/data-processing-terms" target="_blank" rel="noopener noreferrer">{ "Google Cloud Data Processing Terms" }</a>
                            { "." }
                        </p>
                        <p>{ "Eine weitergehende Weitergabe findet nicht statt." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "5. Speicherung und Löschung" }</h2>
                        <ul>
                            <li>{ "Server-Logfiles: Löschung spätestens nach 30 Tagen." }</li>
                            <li>{ "Nutzerdaten (E-Mail, letzter Login, Anzahl Anfragen): Speicherung bis zur Löschung des Nutzerkontos bzw. solange dies zur Bereitstellung der App erforderlich ist." }</li>
                            <li>{ "Backups: Datensicherungen werden rollierend geführt und spätestens nach 30 Tagen automatisiert überschrieben." }</li>
                            <li>{ "E-Mail-Kommunikation: Löschung nach Abschluss der Anfrage, sofern keine gesetzlichen Aufbewahrungspflichten bestehen." }</li>
                            <li>{ "Bei Löschung eines Nutzerkontos werden die zugehörigen personenbezogenen Daten innerhalb von 7 Tagen entfernt bzw. anonymisiert." }</li>
                        </ul>
                    </div>

                    <div class="legal-block">
                        <h2>{ "6. Ihre Rechte" }</h2>
                        <p>{ "Sie haben das Recht auf Auskunft, Berichtigung, Löschung, Einschränkung der Verarbeitung sowie Datenübertragbarkeit. Außerdem steht Ihnen ein Widerspruchsrecht gegen die Verarbeitung personenbezogener Daten zu, die wir auf Basis berechtigter Interessen verarbeiten. Eine erteilte Einwilligung können Sie jederzeit mit Wirkung für die Zukunft widerrufen." }</p>
                        <p>{ "Wenden Sie sich hierzu bitte an die oben genannte verantwortliche Stelle. Ihnen steht zudem ein Beschwerderecht bei einer Datenschutzaufsichtsbehörde zu." }</p>
                        <p class="legal-note">
                            { "Zuständige Aufsichtsbehörde: Der Landesbeauftragte für den Datenschutz und die Informationsfreiheit Baden-Württemberg (LfDI BW), Lautenschlagerstraße 20, 70173 Stuttgart – " }
                            <a href="https://www.baden-wuerttemberg.datenschutz.de" target="_blank" rel="noopener noreferrer">{ "https://www.baden-wuerttemberg.datenschutz.de" }</a>
                        </p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "7. Datensicherheit" }</h2>
                        <p>{ "Wir setzen technische und organisatorische Maßnahmen ein, um Ihre Daten gegen Manipulation, Verlust, Zerstörung oder unbefugten Zugriff zu schützen und passen diese fortlaufend an den Stand der Technik an." }</p>
                        <p>{ "Die Übertragung personenbezogener Daten erfolgt ausschließlich über verschlüsselte Verbindungen (HTTPS/TLS)." }</p>
                    </div>

                    <div class="legal-block">
                        <h2>{ "8. Aktualität und Änderung dieser Datenschutzerklärung" }</h2>
                        <p>{ "Wir behalten uns vor, diese Datenschutzerklärung zu aktualisieren, um sie an geänderte Rechtslagen oder Änderungen des Dienstes anzupassen." }</p>
                    </div>

                    <p class="legal-note">{ "Stand: 10. November 2025" }</p>

                    <LegalLinks />
                </section>
            </main>
        </div>
    }
}
