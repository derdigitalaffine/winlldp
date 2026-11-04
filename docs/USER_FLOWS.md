# Bedienung

## Empfang
Startet mit verbundenen Adaptern, sofern Npcap Zugriff erlaubt. Unter Adapter Auswahl ändern und anwenden. Übersicht zeigt Switch/Port, Suche, Details und sämtliche TLV-Rohdaten. Ohne Treiber PCAP öffnen oder Npcap separat installieren. In Einstellungen bei Bedarf bewusst mit Administrator- oder Standardzugriff neu starten. Keine automatische Erhöhung.

## Sender
Eigenen Bereich öffnen, Adapter wählen, Angaben bearbeiten. Windowsname, MAC und Adapterbeschreibung sind vorbelegt. Zusatz-TLVs als Typ/Hex eintragen oder Vorlage hinzufügen. Die Paketvorschau und Validierung helfen vor dem bewussten Einschalten. Zum Ändern eines aktiven Profils zuerst stoppen. Ein Sender bleibt im Tray aktiv, nach vollständigem Neustart ist er immer aus. Beim Stop wird ein LLDP-Shutdown versucht.

## Protokoll und Dateien
Protokoll enthält Entdeckung/Änderung/TTL-Ablauf sowie Capture-/Sendestatus. Historie ist lokal, begrenzt und löschbar. CSV/JSON exportieren Nachbarn (JSON zusätzlich Ereignisse), PCAP exportiert die letzten gepufferten LLDP-Frames. PCAP-Import ist Offlineanalyse mit Zeitstempeln der Datei; Offline-Nachbarn werden nicht anhand der aktuellen Uhr entfernt. Fremde Nicht-LLDP-Pakete werden verworfen. PCAPNG gegebenenfalls in Wireshark in klassischen Ethernet-PCAP umwandeln.

## Fenster und Design
Schließen minimiert standardmäßig ins Tray. Doppelklick auf Otter oder Öffnen im Traymenü bringt das Fenster zurück. Über Beenden vollständig schließen. Ohne funktionsfähiges Tray wird das Schließen nicht abgefangen. Die Einstellung erlaubt direktes Beenden beim Schließen. Deutsch/English/Nederlands und System/Hell/Dunkel unter Einstellungen.

## Über
Otter, Version, Dominik Tröster / Verbandsgemeinde Otterbach-Otterberg, MIT-Lizenz und Drittanbieterhinweise.

## CLI
`WinLLDP.exe --cli --help` zeigt die Optionen. Weil die Haupt-EXE eine GUI-Anwendung ist, in cmd.exe mit `start /wait "" WinLLDP.exe --cli ...` aufrufen. Für normale Konsolenausgabe und Pipes liegt zusätzlich `WinLLDP-CLI.exe` bei.

Beispiele:
```powershell
.\WinLLDP-CLI.exe --list-adapters
.\WinLLDP-CLI.exe --adapter Ethernet --duration 30 --output neighbors.json
.\WinLLDP-CLI.exe --read capture.pcap --format csv --output neighbors.csv
.\WinLLDP-CLI.exe --adapter Ethernet --write-profile sender.json
.\WinLLDP-CLI.exe --adapter Ethernet --send sender.json --duration 120 --output neighbors.json
```
Diagnosen auf stderr, Textdaten auf stdout oder in `--output`; PCAP benötigt eine Ausgabedatei. Exitcode 0 bei Erfolg, 2 bei Fehlern. Senden endet beim CLI-Ende mit Shutdown-Versuch.
