# Windows-Build und Release

## Voraussetzungen
- Windows 10/11 x64 mit Visual Studio 2022 C++-Buildtools und Windows SDK, oder GitHub Actions `windows-2022`.
- Aktuelle stabile Rust-Toolchain mit Target `x86_64-pc-windows-msvc`.
- Inno Setup 6 für die Installationsdatei.
- Für die **Kompilierung** ist kein installiertes Npcap erforderlich: die Bibliothek wird bei Bedarf dynamisch geladen. Der native Windows-Packet-Monitor-Empfang wird als Rust-Abhängigkeit eingebunden.
- LLDP-Live-Empfang verwendet standardmäßig den in Windows enthaltenen Packet Monitor (mit Administratorrechten). Npcap ist optional für Empfang/Fallback und erforderlich für aktives LLDP-Senden.

## Lokaler Build (PowerShell)

```powershell
rustup target add x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc --bins
pwsh -File scripts/package_windows.ps1
```

Der Packaging-Schritt startet die GUI mit `--self-test`, prüft `winlldp-cli.exe --help`, erzeugt mit Inno Setup das Installationspaket und legt portable EXEs, Lizenzhinweise, `BUILD_INFO.txt` und `SHA256SUMS.txt` unter `build/` ab. Bei einem Fehler wird die Veröffentlichung abgebrochen; nicht erfolgreiche Builds werden nicht als Releases ausgegeben.

## CI und Veröffentlichung
Der Workflow `.github/workflows/windows-release.yml` führt Tests, Build, Smoke-Checks und Packaging auf Windows aus. Für den Release-Branch erfolgt nur der Action-Artefakt-Upload. Nach einem erfolgreichen Merge nach `main` wird aus dem auf dem Runner gebauten `build/`-Verzeichnis ein Commit erzeugt, dann das unveränderliche Tag `v1.1.0` gesetzt und das Release mit genau diesen Dateien veröffentlicht. Existiert das Release bereits, wird es nicht stillschweigend überschrieben. Die Workflow-Berechtigung `contents: write` und verfügbare Windows-GitHub-Actions-Runner sind dafür Voraussetzung.

Für weitere Versionen sind die Versionsangaben in Cargo.toml, Inno-Skript, Packaging-Skript und Workflow zu erhöhen. Alte Tags nie verschieben.

## Prüfumfang und Grenzen
- Unit-/Integrationstests: Parser und Export, Nachbarzustände, Sendeprofil, Einstellungen gemäß `docs/ACCEPTANCE_TESTS.md`.
- Windows CI: kompilierte MSVC-GUI und CLI, GUI-Selbsttest und CLI-Aufruf.
- **Nicht durch CI abgenommen:** reale LLDP-Verbindungen an physischen Switches, Npcap-Berechtigungen/UAC, Windows 10/11 auf Endgeräten, Anzeige und Usability, echter Installer-/Uninstallerdurchlauf.
- Windows-Programme sind **nicht signiert**. Portable bedeutet ohne Installer kopierbar. Der Empfang benötigt keinen zusätzlich installierten Treiber, aber Windows Packet Monitor und Administratorrechte; der Sender erfordert Npcap.
