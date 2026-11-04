# WinLLDP

WinLLDP ist ein nativer Windows-LLDP-Client für Windows 10/11 x64 mit Rust/egui-Oberfläche, CLI und optionalem LLDP-Sendebetrieb innerhalb der Anwendung. WinLLDP installiert keinen eigenen Windows-Dienst.

## Funktionen
- LLDP-Nachbarn pro Adapter anzeigen: Switch, Port, Chassis, Fähigkeiten und sämtliche TLVs einschließlich Rohdaten.
- Live-Empfang über einen separat installierten Npcap-Treiber; ohne Npcap stehen Offline-PCAP und Adapterdiagnose bereit.
- Sender startet **niemals automatisch**. Bewusste Aktivierung auf einem ausgewählten Adapter, konfigurierbare Pflichtfelder und weitere TLVs.
- Verlauf der Port-/Nachbarereignisse, CSV-/JSON-/PCAP-Export, Offline-PCAP-Import.
- GUI mit Tray, Deutsch/Englisch/Niederländisch, System-/Hell-/Dunkelmodus.
- CLI als eigenes Konsolenprogramm `winlldp-cli.exe --help` oder über `WinLLDP.exe --cli`.

## Windows-Download
Die Release-Artefakte finden sich unter [GitHub Releases](https://github.com/derdigitalaffine/winlldp/releases) und nach erfolgreichem Windows-Release-Build auch versioniert unter [build/](build/).

- `WinLLDP-1.0.0-Windows-x64.exe`: portable Windows-GUI.
- `winlldp-cli-1.0.0-Windows-x64.exe`: Konsole.
- `WinLLDP-1.0.0-Windows-x64-Setup.exe`: benutzerbezogener Installer mit Deinstallation.
- `SHA256SUMS.txt`: Prüfsummen; `BUILD_INFO.txt`: Rust-/Quellstand und Hinweise.

Die Programme sind unsigniert; Npcap ist separat erforderlich, um live LLDP zu empfangen oder zu senden. Das Installerpaket enthält den Npcap-Treiber **nicht**. Die UI informiert über Zugriff und gegebenenfalls notwendige Administratorrechte. Vor Live-Sendebetrieb die Berechtigung für das betroffene Netzwerk klären.

## Build und QA
Anleitung: [Windows-Build](docs/BUILD_WINDOWS.md). Release- und bekannte Prüfeinschränkungen: [Release Notes v1.0.0](docs/RELEASE_NOTES_v1.0.0.md), [Akzeptanztests](docs/ACCEPTANCE_TESTS.md).

```powershell
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc --bins
pwsh -File scripts/package_windows.ps1
```

Die Pipeline erstellt nur bei erfolgreichen Tests und tatsächlichem Windows-Build ein Release. Echte Windows-/Treiber-/Switch-Tests müssen separat auf Hardware erfolgen und sind durch CI nicht belegt.

## Projekt
- [Spezifikation](docs/PRODUCT_SPEC.md)
- [Architektur und technische Grenzen](docs/ARCHITECTURE.md)
- [Arbeitsorganisation](AGENTS.md)
- [Drittanbieterhinweise](THIRD_PARTY_NOTICES.md)
- [MIT-Lizenz](LICENSE)

Entwickelt von Dominik Tröster, Verbandsgemeinde Otterbach-Otterberg.
