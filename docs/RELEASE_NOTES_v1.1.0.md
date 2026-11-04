# WinLLDP v1.1.0 — LLDP unter Windows ohne Npcap

## Neue Hauptfunktion
**LLDP-Switch- und Portdaten nativ über Windows Packet Monitor empfangen**, ohne Npcap oder zusätzliche Capture-Treiber zu installieren. Erforderlich ist ein unterstützter Windows-10/11-x64-Build mit verfügbaren Packet-Monitor-Schnittstellen und in der Regel ein **als Administrator gestartetes WinLLDP**.

- Empfangsmodus **Automatisch / Windows Packet Monitor / Npcap** im Adapterbereich.
- **Automatisch:** zuerst nativer Windows-Packet-Monitor-Empfang; bei Initialisierungsfehlern Wechsel auf Npcap, sofern installiert und nutzbar.
- Adapterbezogene Auswahl mit eindeutiger Netzwerkinterface-Zuordnung, Filterung auf LLDP EtherType 0x88CC, Erkennung eingehender Frames und Ausschluss lokaler Sende-Echos.
- Anzeige des tatsächlich verwendeten Empfangswegs in der Adaptertabelle, Diagnosefunktion für Windows Packet Monitor, erweiterte CLI-Option `--doctor`.
- Administrator-Neustart über sichtbaren Windows-UAC-Dialog.
- CLI `--backend auto|windows|npcap` für gezielte Tests und Diagnose.
- Weiterhin kompakte Nachbartabelle mit Doppelklick-Detailfenster.

## Bewusst getrennt
- **Empfang:** Windows Packet Monitor, ohne Npcap.
- **LLDP-Senden:** erfordert weiterhin Npcap (separat installiert, explizit aktiviert).
- Offline-PCAP-Import/Export, CSV/JSON-Export, Verlauf, mehrere Adapter, DE/EN/NL und Standard-Tray-Verhalten bleiben erhalten.

## Prüfumfang und bekannte Grenzen
Der Windows-MSVC-CI-Workflow kompiliert und testet GUI und CLI, erzeugt einen Installer, prüft Diagnose-CLI und GUI-Selbsttest und veröffentlicht SHA256SUMS. Ein automatischer Build beweist **nicht**, dass Windows Packet Monitor auf allen Windows-Builds Pakete erfasst. Die verwendete Rust-Bibliothek `winptap` v0.2.0 ist jung; ihre Native-IOCTL-Option verwendet undokumentierte Windows-Schnittstellen. Echte LLDP-Switch-Hardware und UAC-Verhalten müssen auf Zielrechnern zusätzlich getestet werden. Npcap bleibt als Alternative verfügbar.

Mehr: [Windows Packet Monitor](WINDOWS_PACKET_MONITOR.md) und [Npcap-Diagnose](NPCAP_DIAGNOSTICS.md).
