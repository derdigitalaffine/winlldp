# UI-Gestaltungsregeln

## WinLLDP 1.0.1
- Ruhige, responsive Oberfläche für Windows 10/11 mit systemabhängigem hellem/dunklem Design.
- Gemeinsame `surface`-Darstellung für Karten und Einstellungsbereiche mit dezentem Rand, Innenabstand und einem konsistenten Rundungsradius.
- Hierarchie: klare Seitentitel, zurückhaltende Beschreibungen, gut lesbare KPI-Zahlen und kompakte Tabellenköpfe.
- Arbeitsbereiche mit gleichmäßigen Abständen; Kopfzeile, Navigation und Statusleiste bleiben sichtbar.

## Nachbartabelle
- **Eine Zeile pro Nachbar**, keine 78 Pixel hohen Karten.
- Fünf Spalten: Switch, Port, Adapter, angekündigte TTL in Sekunden, zuletzt gesehen in UTC.
- Jedes Feld in einer Zeile ist zur Auswahl klickbar. Vollständige Nachbardetails/TLV-Rohdaten erscheinen unterhalb.
- Horizontaler Tabellen-Scroll bei schmalen Fenstern, begrenzte Tabellenhöhe mit vertikalem Scroll für viele Nachbarn.
- Filter ohne Datenverlust auf Switch, Port, Chassis-ID, Adapter und Quell-MAC; Zähler `Treffer / Gesamt`.
- Trilingual beschriftete Tabelle und Suchzustände, klare Auswahlhervorhebung auch ohne Farbe.

## Zu prüfen
- Windows-Skalierung 100/150/200 %, Tastaturfokus und screenreader-gerechte Tabellenbedienung.
- Lange Port- und Adapterwerte, viele Nachbarn, Leerzustände, Offline-PCAP, drei Sprachen und Hell-/Dunkelmodus.
- CI bestätigt Compilation und native Smoke-Checks, ersetzt aber keine visuelle Hardwareabnahme.
