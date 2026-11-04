# Abnahmekriterien

## Automatisiert
- Parser: Roundtrip, unbekannte TLVs, VLAN/QinQ, Abschneiden und ungültige Daten, Pflichtreihenfolge.
- Sender: Pflichtfelder, Typ-/Längen-/MTU-Grenzen, Shutdown mit TTL 0.
- Nachbarn: Adapterisolation, TTL-Ablauf und Shutdown.
- Export: PCAP-Roundtrip/Trunkierung, CSV-Formelschutz, JSON-Struktur.
- Einstellungen: Sprache/Thema/Tray persistieren, Senden bleibt nach Neustart aus.
- Release: Windows-PE-Binary, Icon/Version, Start im Wine-Ersatztest, Installer-/Uninstallerpfad.

## Reale Windows-/Netzwerkabnahme
Auf Windows 10 und 11 mit physischem Ethernet und LLDP-fähigem Switch prüfen:
1. Npcap fehlt: verständlicher Status, Offlineimport verfügbar, kein erfundener Nachbar.
2. Normaler Zugriff und Admin-only-Npcap: Zugriff bzw. nachvollziehbare UAC-Umschaltung.
3. Switch/Port und vollständige TLVs; mehrere Adapter, Kabelziehen und TTL-Ablauf.
4. Sender ist zunächst aus. Bewusster Start, kontrollierter Inhalt am Switch/Wireshark, Stop/TTL-0.
5. Tray erhält aktiven Sender; vollständiges Ende und Neustart lassen Senden aus.
6. Deutsch/Englisch/Niederländisch, hell/dunkel/System, Windows-Skalierung 100/150/200 %.
7. CSV, JSON, PCAP öffnen; fremde und abgeschnittene Dateien robust behandeln.
8. Installation/Verknüpfungen/Deinstallation im normalen Benutzerkonto.

Ohne echte Windows-/Switch-Hardware bleiben diese Hardwareprüfungen ausdrücklich unbestätigt. Wine ist ein Ersatztest, kein vollständiger Windows-Nachweis.
