# WinLLDP v1.0.0 — Windows x64

Erste Windows-Veröffentlichung für Windows 10 und Windows 11 (x64).

## Inhalt
- Native, dreisprachige Oberfläche (Deutsch, Englisch, Niederländisch), Tray und helles/dunkles Design.
- LLDP-Empfang über vorhandenes Npcap mit Adapter-, Switch-, Port- und TLV-Details.
- Bearbeitbares, standardmäßig ausgeschaltetes LLDP-Senden innerhalb der Anwendung, ohne eigenen Windows-Dienst.
- Änderungsprotokoll, Offline-PCAP-Import, JSON-/CSV-/PCAP-Export und CLI.
- Portable GUI-EXE, separate CLI-EXE sowie Installer für das aktuelle Benutzerkonto.

## Downloads
- `WinLLDP-1.0.0-Windows-x64.exe`: portable GUI.
- `winlldp-cli-1.0.0-Windows-x64.exe`: Konsole / Automatisierung.
- `WinLLDP-1.0.0-Windows-x64-Setup.exe`: Inno-Setup-Installer (mit Deinstallation).
- `SHA256SUMS.txt`, `BUILD_INFO.txt`, Lizenz- und Dependency-Hinweise.

## Voraussetzungen und Sicherheit
Npcap ist **nicht** im Download enthalten. Live-LLDP-Erfassung und Sendebetrieb erfordern einen separat installierten, passenden Npcap-Treiber; Zugriffsrechte hängen von der Npcap-Konfiguration ab. Ohne Npcap bleibt die Offline-PCAP-Analyse möglich.

**Achtung:** Die EXE-Dateien sind nicht codesigniert. Der optionale LLDP-Sender wird nicht automatisch gestartet und sollte nur auf autorisierten Netzwerken aktiviert werden.

Der CI-Lauf testet Rust-Komponenten auf einem Windows-Runner, kompiliert die EXE-Dateien, startet den GUI-Selbsttest und prüft die CLI. Ein realer Netzwerkswitch, Windows-10-/11-Endgeräte mit Npcap und der vollständige Installer-/Uninstallerablauf werden dadurch **nicht** ersetzt; die reale Hardwareabnahme bleibt offen. Herkunft, Buildumgebung und SHA-256-Prüfsummen sind den mitgelieferten Dateien zu entnehmen.

Quellcode: https://github.com/derdigitalaffine/winlldp
