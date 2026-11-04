# WinLLDP

WinLLDP ist ein nativer Windows-LLDP-Client für Windows 10/11 x64 mit Rust/egui-Oberfläche, CLI und optionalem LLDP-Sendebetrieb innerhalb der Anwendung. WinLLDP installiert keinen eigenen Windows-Dienst.

## Funktionen
- LLDP-Nachbarn pro Adapter anzeigen: Switch, Port, Chassis, Fähigkeiten und sämtliche TLVs einschließlich Rohdaten.
- **Treiberfreier LLDP-Live-Empfang** über den in Windows integrierten Packet Monitor (Administratorrechte erforderlich), mit Npcap als optionalem Empfangs-Fallback. Ohne Npcap funktionieren Empfang, PCAP-Offlineanalyse und Diagnose.
- Sender startet **niemals automatisch**. Bewusste Aktivierung auf einem ausgewählten Adapter, konfigurierbare Pflichtfelder und weitere TLVs.
- Verlauf der Port-/Nachbarereignisse, CSV-/JSON-/PCAP-Export, Offline-PCAP-Import.
- **Kompakte GUI** ohne Seitenleiste: Nachbarn, Adapter und Senden im Hauptbereich; Protokoll, Einstellungen und Info im Menü **Mehr**. Einfache Suchleiste, ein Datei-Menü und Tabellen mit Doppelklick-Details.
- GUI mit Tray, Deutsch/Englisch/Niederländisch, System-/Hell-/Dunkelmodus.
- CLI als eigenes Konsolenprogramm `winlldp-cli.exe --help` oder über `WinLLDP.exe --cli`.

## Windows-Download
Die Release-Artefakte finden sich unter [GitHub Releases](https://github.com/derdigitalaffine/winlldp/releases) und nach erfolgreichem Windows-Release-Build auch versioniert unter [build/](build/).

- `WinLLDP-1.2.0-Windows-x64.exe`: portable Windows-GUI.
- `winlldp-cli-1.2.0-Windows-x64.exe`: Konsole.
- `WinLLDP-1.2.0-Windows-x64-Setup.exe`: benutzerbezogener Installer mit Deinstallation.
- `SHA256SUMS.txt`: Prüfsummen; `BUILD_INFO.txt`: Rust-/Quellstand und Hinweise.

Die Programme sind unsigniert. Für den LLDP-Empfang wird **kein Npcap benötigt**, wenn Windows Packet Monitor mit Administratorrechten zur Verfügung steht. Für das Senden wird Npcap weiterhin separat benötigt. Das Installerpaket enthält den Npcap-Treiber **nicht**. Unter **Adapter** lässt sich der Empfangsweg **Automatisch / Windows Packet Monitor / Npcap** auswählen. Die UI zeigt den tatsächlich verwendeten Empfangsweg sowie konkrete Fehler. Unter **Adapter → Diagnose und Npcap** lässt sich die offizielle Npcap-Downloadseite öffnen und der heruntergeladene Setup-Assistent aus WinLLDP mit UAC starten. Anschließend Treiber erneut prüfen und Empfang neu starten. Für Admin-only-Npcap kann WinLLDP bewusst erhöht neu gestartet werden. Vor Live-Sendebetrieb die Berechtigung für das betroffene Netzwerk klären.

**Hinweis:** Der kostenlose Npcap-Installer wird aus Lizenzgründen weder gebündelt noch unbemerkt heruntergeladen oder installiert. Den signierten Installer direkt von npcap.com beziehen und dessen Lizenz prüfen. Details: [Windows-Empfang ohne Npcap](docs/WINDOWS_PACKET_MONITOR.md) und [Npcap-Diagnose und Installation](docs/NPCAP_DIAGNOSTICS.md).

## Build und QA
Anleitung: [Windows-Build](docs/BUILD_WINDOWS.md). Release- und bekannte Prüfeinschränkungen: [Release Notes v1.2.0](docs/RELEASE_NOTES_v1.2.0.md), [Akzeptanztests](docs/ACCEPTANCE_TESTS.md).

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
