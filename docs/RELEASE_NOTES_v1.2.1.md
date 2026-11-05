# WinLLDP v1.2.1 — vollständige Lizenzpakete

This patch release regenerates the third-party license bundle from the locked Linux and Windows dependency trees during CI. The full notices are included in the source repository, portable Windows and Linux packages, installer, checksums and release assets.

The compact Slint UI, Windows 10/11 client, CLI and Linux PCAP-analysis build are unchanged from v1.2.0.

## Downloads

- `WinLLDP-1.2.1-Windows-x64.exe` — portable GUI
- `winlldp-cli-1.2.1-Windows-x64.exe` — Windows CLI
- `WinLLDP-1.2.1-Windows-x64-Setup.exe` — per-user installer
- `WinLLDP-Linux-x64` — native Linux GUI
- `WinLLDP-CLI-Linux-x64` — Linux CLI

Verify the SHA-256 values in `SHA256SUMS.txt` and `SHA256SUMS-Linux.txt` before use. Windows live capture via Packet Monitor requires administrator rights; LLDP sending requires separately installed Npcap. Linux currently supports offline PCAP analysis and CLI operation.
