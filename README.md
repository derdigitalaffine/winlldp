# WinLLDP

Kompakter LLDP-Client für Windows 10/11 und Linux x64, mit einer gemeinsamen nativen Slint-Oberfläche. WinLLDP zeigt Switch, Port und sämtliche LLDP-TLVs samt Rohdaten.

## Funktionen

- Live-Empfang auf Windows über Packet Monitor oder optional Npcap; Empfangsweg und Adapter sind in der UI wählbar.
- LLDP-Senden ist standardmäßig aus, läuft in der Anwendung ohne eigenen Dienst und benötigt Npcap.
- PCAP importieren und JSON, CSV oder PCAP exportieren; Verlauf lokal speichern.
- GUI und CLI (`--cli`) teilen Parser, Capture und Export. Linux-Build: PCAP-Auswertung plus CLI; Live-Capture/Senden sind Windows-Funktionen.
- Deutsch, Englisch, Niederländisch; kompakte helle/dunkle Slint-Oberfläche, Tray unter Windows.

## Downloads

Fertige Programme und Prüfsummen sind in [`build/`](build/) sowie [GitHub Releases](https://github.com/derdigitalaffine/winlldp/releases) abgelegt. Windows- und Linux-x64-Builds werden aus demselben Commit erstellt. Windows-Liveempfang über Packet Monitor erfordert Administratorrechte; optionales Npcap ist für Senden erforderlich.

- Windows: portable GUI, Konsole und per-user Setup.
- Linux x64: `WinLLDP-Linux-x64` und `WinLLDP-CLI-Linux-x64`.

```sh
./WinLLDP-Linux-x64
./WinLLDP-CLI-Linux-x64 --read capture.pcap --format json
```

## Build

Windows: Visual Studio 2022 C++-Buildtools, Windows SDK, Inno Setup 6; dann `cargo test --locked --target x86_64-pc-windows-msvc`, `cargo build --locked --release --target x86_64-pc-windows-msvc --bins` und `pwsh -File scripts/package_windows.ps1`.

Linux: Rust stable und Slint-Abhängigkeiten; `sh scripts/build-linux.sh`.

## Lizenz

WinLLDP-Quellcode wird unter MIT und GPL-3.0-only angeboten. Die ausgelieferten Slint-Binaries verwenden die GPL-3.0-only-Lizenz des Toolkits. Lizenz- und Herkunftshinweise liegen in `LICENSE`, `LICENSE-GPL-3.0-only.txt`, `THIRD_PARTY_NOTICES.md` und `THIRD_PARTY_LICENSES.txt`; CI erzeugt die vollständige Abhängigkeitsliste aus `Cargo.lock`.

Entwickelt von Dominik Tröster, Verbandsgemeinde Otterbach-Otterberg.
