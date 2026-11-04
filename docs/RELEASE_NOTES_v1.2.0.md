# WinLLDP v1.2.0 — kompakter, einheitlicher Arbeitsbereich

WinLLDP konzentriert sich wieder auf das Wesentliche: **Nachbarn erkennen, Adapter auswählen und LLDP bei Bedarf senden.**

## Oberfläche
- Bisherige sechs Punkte in der großen Seitenleiste ersetzt durch drei klare Tabs und ein schlankes **Mehr**-Menü für Protokoll/Einstellungen/Info/Beenden.
- Ein einheitlicher Start-/Stopp-Schalter in der Kopfzeile; dauerhaft schmale Statusleiste; keine redundanten Statusblöcke oder Dashboard-Kacheln.
- Nachbartabelle mit Suche und einem **Datei**-Menü für Import/Export. Doppelklick/Enter öffnet den kompakten Detaildialog einschließlich aller Roh-TLVs.
- Adapter in einer übersichtlichen Status-Tabelle; geänderte Auswahl erst mit **Auswahl übernehmen** aktivieren. **Diagnose und Npcap** standardmäßig ausgeblendet; Adminrechte kontextabhängig sichtbar.
- **Senden:** ein sicherer, zweistufiger Ablauf. „Sender vorbereiten“ richtet nur Npcap und die Auswahl ein; zum tatsächlichen Senden ist „Senden einschalten“ nötig. Nur drei Hauptfelder dauerhaft sichtbar; technische Einstellungen und zusätzliche TLVs aufklappbar.
- Aktivitätsprotokoll als kompakte Tabelle mit Doppelklick-Details und Löschbestätigung. Einstellungen konsolidiert.
- Konsistente Abstände, Schriftgrößen, Farben, Oberflächen und Controls. Standardfenster 860 × 550, Mindestfenster 680 × 430.

## Weitere Verbesserungen
- **Adapterauswahl bleibt über Neustarts gespeichert**, einschließlich ausdrücklich leerer Auswahl; bisherige Konfigurationsdateien bleiben lesbar.
- Die LLDP-Empfangswege aus v1.1.0 bleiben erhalten: Windows Packet Monitor ohne Npcap als bevorzugter Empfang, Npcap-Fallback und Npcap-only für aktives LLDP-Senden.
- Alle drei Sprachen, System-/Hell-/Dunkelmodus, Tray, Offline-PCAP, CLI und Datenexport bleiben verfügbar.

## Qualität und Grenzen
Windows MSVC-Unit-Tests, Release-Build, GUI-Selbsttest, CLI-Diagnose, Inno-Setup-Paketierung und SHA256-Listen werden vor dem Release in GitHub Actions überprüft. Die automatisierte Prüfung kann Bedienbarkeit auf echten Windows-Displays, verschiedene DPI-Skalierungen und den physischen LLDP-Switch-Empfang nicht belegen. Das Release ist weiterhin unsigniert; Npcap wird nicht mitgeliefert.

Dokumentation: [Oberfläche](DESIGN_SYSTEM.md), [Bedienung](USER_FLOWS.md) und [Windows Packet Monitor](WINDOWS_PACKET_MONITOR.md).
