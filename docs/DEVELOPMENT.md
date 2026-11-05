# Entwickeln und bauen

## Slint-Oberfläche
Die gemeinsame Desktopoberfläche liegt in `ui/app-window.slint` und wird während des Rust-Builds kompiliert. Linux unterstützt die Offline-PCAP-Analyse und CLI; Liveempfang und Senden werden auf Windows angeboten.

## Linux x64
Rust stable und die nativen X11/Wayland-Buildabhängigkeiten bereitstellen, dann:

```sh
sh scripts/build-linux.sh
```

Das Script formatiert, testet, prüft Clippy und erzeugt zwei echte Binaries unter `build/linux-x64/`.

## Windows x64
Unter Windows 10/11: Rust stable (MSVC), Visual Studio C++-Buildtools, Windows SDK und Inno Setup 6. Npcap ist für den Build nicht notwendig.

```powershell
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc --bins
pwsh -File scripts/package_windows.ps1
```

Das Setup ist eine Inno-Setup-Datei pro Benutzer. Packet Monitor ist unter Windows nativ; Npcap bleibt optional für Empfang und Voraussetzung für LLDP-Senden.

## Drittanbieter und Lizenz
`python scripts/collect_licenses.py` aktualisiert Abhängigkeitstexte. WinLLDP-Quellcode ist unter MIT und GPL-3.0-only verfügbar. Das Slint-Binary verwendet GPL-3.0-only. Beide vollständigen Lizenztexte und Notices werden mit ausgeliefert.

## CI und Abnahme
`.github/workflows/windows-release.yml` baut Windows plus Linux auf jedem PR und erzeugt nach erfolgreichem Merge Build-Artefakte, Prüfsummen und Release. Die vollständigen Drittanbieter-Lizenzen werden im Linux-Build aus `Cargo.lock` gesammelt und in die Windows-/Linux-Pakete kopiert. CI ersetzt keine echte Abnahme unter Windows 10/11, mit Packet Monitor, Npcap und Switch-Hardware; diese Grenzen bleiben in den Release Notes dokumentiert.
