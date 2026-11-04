# AGENTS.md

## Mission
Entwickle WinLLDP zu einem hochwertigen Windows-LLDP-Client mit optionalem LLDP-Sendebetrieb innerhalb der laufenden Anwendung, ohne Windows-Dienst. Liefere eine fertig kompilierte portable EXE und ein Installationspaket im Repository.

## Herkunft
Diese Arbeitsorganisation übernimmt die Struktur aus der AGENTS.md von derdigitalaffine/kostuemwettbewerb (Stand c9e99775decb2b35af8def14ae366c3fe69d6d00).
Übernommen werden Spezifikation als gemeinsame Grundlage, Sicherheitsinvarianten, kleine nachvollziehbare Arbeitspakete, Akzeptanztests, Dokumentation und überprüfbare Fertigstellung.
Halloween-, Django-, PostgreSQL- und Veranstaltungsanforderungen gelten hier nicht.

## Zuerst lesen
1. README.md
2. docs/PRODUCT_SPEC.md
3. Alle vorhandenen relevanten Architektur-, Sicherheits-, UX-, Design-, Roadmap- und Akzeptanzdokumente unter docs/
4. Zugehörige Issues und zentrales Tracking-Issue, sobald vorhanden

Das Repository ist die gemeinsame fachliche Grundlage. Konflikte zwischen Issues, Dokumentation und Implementierung untersuchen und Entscheidungen dokumentieren.
Das Interview ist abgeschlossen. Der Nutzer hat autonome Fertigstellung und vier nach main zu mergende PRs beauftragt. Offene Details robust selbst entscheiden und dokumentieren.

## Arbeitsorganisation
- Produkt und Anforderungen: Interviewergebnisse, offene Fragen, Prioritäten und Akzeptanzkriterien pflegen.
- Architektur und Implementierung: einfachste robuste Lösung für die vereinbarten Windows-Zielsysteme wählen.
- UX/UI: verständliche Oberfläche, klare Zustände, Tastaturbedienung und hilfreiche Fehlerbehandlung.
- Sicherheit: Netzwerkdaten als nicht vertrauenswürdig behandeln; Rechte und Abhängigkeiten minimieren.
- QA: Kernabläufe, Fehlersituationen und Release-Artefakte nachweisbar prüfen.
- Build und Auslieferung: portable EXE und Installationspaket tatsächlich kompilieren und im Repository bereitstellen.
Diese Verantwortlichkeiten sind keine Behauptung, dass mehrere Agenten gestartet wurden.

## Technische Vorgaben
- LLDP-Empfang mit Anzeige des angeschlossenen Switches und Ports.
- Optionaler LLDP-Sendebetrieb in der Anwendung; kein dafür laufender Windows-Dienst.
- Portable EXE und Installationspaket als vereinbarte Lieferformate.
- Windows 10 und Windows 11 unterstützen; CPU-Architekturen noch klären.
- Betrieb ohne Administratorrechte sowie mit Administratorrechten/Npcap komfortabel über die UI umschaltbar machen. Fähigkeiten und Voraussetzungen beider Modi technisch nachweisen und transparent darstellen.
- Rust/egui, Windows x64, Npcap dynamisch; Deutsch/Englisch/Niederländisch.
- Keine Architektur oder Treiberfreiheit aus dem Namen „portable“ ableiten.
- Netzwerkwerbung nur bewusst aktivieren; Zustand und betroffene Adapter sichtbar anzeigen.

## Sicherheits- und Datenschutzinvarianten
- Keine echten Secrets, Zugangsdaten oder privaten Netzwerkmitschnitte committen.
- Empfangene Frames strikt und begrenzt parsen, einschließlich ungültiger TLV-Längen.
- Netzwerkdaten niemals als Code oder unbereinigtes Markup ausführen.
- Erhöhte Rechte nur dort verlangen, wo technisch notwendig und dokumentiert.
- Ohne explizite Produktentscheidung keine Telemetrie oder Übertragung an externe Dienste.
- Diagnosedaten und Exporte auf sensible Netzwerkdetails prüfen.
- Sendebetrieb beim Beenden der Anwendung sauber beenden.

## Design
- Höchste Priorität für eine hochwertige, komfortable UI.
- Klar lesbare Adapter-, Nachbar-, Switch- und Portinformationen; alle empfangenen LLDP-Daten müssen zugänglich sein, einschließlich nicht interpretierter Erweiterungen.
- Lade-, Empfangs-, Sende-, Leer- und Fehlerzustände verständlich unterscheiden.
- Wichtige Informationen nicht ausschließlich durch Farbe vermitteln.
- Systemabhängiges helles/dunkles Design, manuell überschreibbar.

## Tests
Jede Funktion benötigt die kleinsten sinnvollen Prüfungen ihrer Akzeptanzkriterien.
Mindestens Parser-Grenzfälle, Nachbarablauf/TTL, Adapterzustände sowie Start/Stop des Sendebetriebs prüfen, sobald implementiert.
Synthetische Frames als Fixtures verwenden.
Echte Windows-, Treiber- und Netzwerkprüfungen klar von lokalen Ersatztests unterscheiden.
Keinen erfolgreichen Test behaupten, der nicht ausgeführt wurde.

## Git- und Aufgabenworkflow
Kleine kohärente Änderungen mit verständlichen Commits; Feature-Branch und PR passend zum Repository-Workflow.
Aus den bestätigten Anforderungen ausführbare Issues mit Abhängigkeiten und Akzeptanzkriterien sowie ein zentrales Tracking ableiten.
Issues erst nach nachgewiesener Erfüllung schließen.
Änderungsbeschreibungen nennen Verhalten, Konfiguration, Abhängigkeiten, durchgeführte Tests und verbleibende Einschränkungen.

## Externe Limits
Fehlendes GitHub-Actions-Budget ist kein automatischer Stop-Grund.
Prüfungen und Builds soweit möglich lokal oder über einen geeigneten verfügbaren Buildpfad durchführen.
Nicht ausführbare Prüfungen mit Grund, Ersatzprüfung und verbleibender Unsicherheit dokumentieren.
Ein fehlender Windows-Build darf nicht als erfolgreich erstellt ausgegeben werden.

## Definition of Done
- Vereinbarte Akzeptanzkriterien und Kernabläufe erfüllt.
- Sicherheits- und Fehlersituationen geprüft.
- Portable EXE und Installationspaket erfolgreich kompiliert und unter build/ im Repository versioniert.
- Builddateien sind echte Binärartefakte, keine Platzhalter und keine bloßen Downloadlinks.
- Dokumentierter reproduzierbarer Build, Versions-/Quellstand, Prüfsummen und Abhängigkeiten.
- Windows-Start und Installation/Deinstallation geprüft oder konkrete ausstehende Prüfungen klar benannt.
- README, Betriebs-/Entwicklerhinweise und QA-Bericht entsprechen dem tatsächlichen Stand.
