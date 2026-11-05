# WinLLDP v1.2.0 — Slint UI and Linux x64

## Änderungen
- Eine gemeinsame, kompakte Slint-Oberfläche ersetzt egui unter Windows und Linux.
- Linux x64 erhält GUI und eigenständige CLI für PCAP-Auswertung, JSON/CSV/PCAP-Exporte und Protokolltests.
- Windows Packet Monitor / Npcap und der konfigurierbare Sendeweg bleiben auf Windows verfügbar. Adapter, Empfangsweg und Ausgabe können in der Slint-Oberfläche gewählt werden.
- Sprache DE/EN/NL, heller/dunkler/systembasierter Modus, lokaler Verlauf und Windows-Tray bleiben Bestandteil.
- Der Windows- und Linux-Build wird im selben Workflow geprüft und als Dateien unter `build/` sowie als Release ausgeliefert.

## Nutzung
Linux: `WinLLDP-Linux-x64` öffnet die Oberfläche; `WinLLDP-CLI-Linux-x64 --read file.pcap --format json` liest PCAP und schreibt JSON. Live-Capture und Senden werden weiterhin nur unter Windows unterstützt.

Windows: Packet Monitor ist der native Empfangsweg; Administratorrechte können erforderlich sein. Npcap ist optional für Empfang und nötig für den Sendeweg. Kein Treiber wird mitgeliefert.

## Lizenz
WinLLDP-Quellcode: MIT und GPL-3.0-only. Slint-Programme werden hier mit der Slint-GPL-3.0-only-Lizenz ausgeliefert. Vollständige Texte und Abhängigkeitsnachweise sind im Build enthalten.

CI-Builds und Tests belegen keine physische Switch-/Npcap-/Packet-Monitor-Abnahme unter allen Windows-Versionen. Die ausführbaren Windows-Dateien bleiben unsigniert.
