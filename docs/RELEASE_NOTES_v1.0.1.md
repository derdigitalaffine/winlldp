# WinLLDP v1.0.1 — kompakte Nachbartabelle und UI-Polish

Dieses Update für Windows 10/11 x64 verbessert die Bedienbarkeit der Netzwerkübersicht und vereinheitlicht das Erscheinungsbild der Anwendung.

## Neu und verbessert
- Die bisherige hohe Neighbor-Kartenliste wurde durch eine kompakte, streifige Tabelle mit **Switch, Port, Adapter, TTL und zuletzt gesehen (UTC)** ersetzt.
- Die Suche filtert nach Switchname, Port-ID und -Beschreibung, Adapter, Chassis-ID und Quell-MAC; Trefferzahl und leere Suchergebnisse sind sichtbar.
- Alle Tabellenspalten sind anklickbar; das selektierte Gerät zeigt seine vollständigen LLDP-TLVs und Rohdaten im Detailbereich darunter.
- Tabelle und Detailbereich haben voneinander getrennte Scrollbereiche, sodass die Ansicht auch mit vielen Nachbarn übersichtlich bleibt.
- Karten, Oberflächen, Trennlinien, Typografie, Abstände, Sidebar und Ereignisprotokoll sind im Hell- und Dunkelmodus konsistenter gestaltet.
- Versionsanzeige in der Sidebar wird automatisch aus der Paketversion erzeugt.

## Windows-Pakete
- `WinLLDP-1.0.1-Windows-x64.exe` – portable Desktop-Anwendung
- `winlldp-cli-1.0.1-Windows-x64.exe` – CLI
- `WinLLDP-1.0.1-Windows-x64-Setup.exe` – Benutzerinstallation (Inno Setup)
- `SHA256SUMS.txt` und `BUILD_INFO.txt` – Prüfsummen und Buildnachweis

## Kompatibilität und Sicherheit
- Windows 10/11 x64, Npcap für Live-LLDP separat installieren. Offline-PCAP-Import funktioniert ohne Capturetreiber.
- Der optionale LLDP-Sender ist weiterhin standardmäßig aus; kein WinLLDP-Windows-Dienst.
- Binaries sind **nicht codesigniert**. Die Pipeline führt Windows-Rust-Tests, MSVC-Release-Build, GUI-Selbsttest, CLI-Aufruf und Installer-Erstellung aus.
- Reale Windows-10-/11-Endgeräte, Switches, Npcap-Berechtigungen und vollständige Installations-/Deinstallationsabläufe bleiben zusätzliche Hardware-/E2E-Abnahmen.
- Gegenüber v1.0.0 sind keine Änderungen am LLDP-Protokollformat beabsichtigt.

Quellcode: https://github.com/derdigitalaffine/winlldp
