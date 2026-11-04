# WinLLDP v1.2.0 – kompakter Arbeitsbereich

## Grundprinzip
Eine kleine Windows-Native-App für drei tägliche Aufgaben: **Nachbarn erkennen**, **Adapter auswählen**, **LLDP senden**. Alles andere ist sekundär. Keine linke Seitenleiste, kein Dashboard mit großen Kennzahlkarten, keine stets geöffneten Diagnose-/Hexblöcke.

- **Kopfzeile:** Logo 27 px, Produktname, drei schlichte Text-Tabs **Nachbarn / Adapter / Senden**, ein einziger primärer **Empfang starten / Stopp**-Schalter und **Mehr**.
- **Mehr:** Protokoll, Einstellungen, Über WinLLDP, Beenden. Der Punkt Beenden beendet vollständig; das normale Fensterschließen respektiert die Tray-Einstellung.
- **Fußzeile:** einzeiliger Live-Status, Anzahl aktiver Adapter und Nachbarn, Senderstatus und letzte Meldung mit ausführlichem Tooltip. Kein Doppelstatus im Inhalt.
- **Nachbarn:** eine Suche, eine einheitliche **Datei**-Aktion mit PCAP-Import und CSV-/JSON-/PCAP-Export, kompakte fünfspaltige Tabelle. Doppelklick/Enter öffnet ein separates Detailfenster mit TLVs.
- **Adapter:** Auswahl als Tabelle. **Aktualisieren**, bei geänderter Auswahl **Auswahl übernehmen**. Empfangsweg als ComboBox; nur bei Bedarf **Diagnose und Npcap** aufklappen. Administrator-Neustart erscheint kontextabhängig.
- **Senden:** Adapter wählen; **Sender vorbereiten** setzt lediglich Npcap und aktiviert den Adapter – sendet nie automatisch. **Senden einschalten** ist eine zweite ausdrückliche Aktion. Drei alltägliche Textfelder; technische IDs, TTL, Intervall, Ziel-MAC, TLVs und Hexvorschau liegen unter ausklappbaren Details.
- **Protokoll:** kompakte dreispaltige Tabelle mit den jüngsten 250 Ereignissen, Doppelklick für vollständige Ereignisdetails; das Löschen braucht eine Bestätigung.
- **Einstellungen:** zwei ComboBoxes für Sprache/Design, zwei Optionen; selten benötigte Administrator-/Datenschutzfunktionen unter einem Abschnitt.

## Visuelles System
- Native egui-Elemente, System-/Hell-/Dunkelmodus, einheitliche Abstände: 7 × 6 px, Schaltflächen 10 × 5 px, Mindesthöhen 27 px, Flächenecken 8 px.
- Typografie: 19 px Überschriften, 14 px Standardtext, 13 px Buttontext, 10–12 px dezente Metadaten.
- Dezente blaue/dunkle Flächen, grünblauer Akzent für Aktivität/Selektion; Primäraktion zum Starten auf dunklem Grün mit weißem Text. Status immer zusätzlich im Klartext.
- Standardfenster **860 × 550**, Mindestfenster **680 × 430**. Tabellenspalten können auf kleinen Displays horizontal scrollen, sekundäre Bereiche vertikal.
- Kein künstlicher LLDP-Nachbar bei Programmstart. Demos nur im expliziten QA-Modus.

## Tastatur und Verantwortung
- Tab für native Widgets, Enter für fokussierte Nachbardetails, Doppelklick für Nachbar- und Protokolldetails, Escape schließt Dialoge.
- Capture-Backend und ausgewählte Adapter bleiben gespeichert, auch bei älteren Konfigurationsdateien. Automatisch eingeschaltetes Senden ist verboten.
- Windows Packet Monitor empfängt mit erhöhten Rechten ohne Npcap; Npcap ist für Senden nötig, aber im Alltag versteckt.

**Reale Darstellung und Interaktion** müssen auf Windows 10/11 mit 100/150/200 % Skalierung überprüft werden; ein CI-Build ersetzt keine visuelle Hardwareabnahme.
