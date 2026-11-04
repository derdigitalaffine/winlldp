# Bedienung v1.2.0

## Nachbarn finden
WinLLDP öffnet den Tab **Nachbarn**. Standardmäßig startet Empfang auf den gespeicherten Adaptern (bei einer alten Installation zunächst auf den verbundenen Adaptern). Rechts oben den einen zentralen **Empfang starten / Stopp**-Schalter verwenden. Die Tabelle zeigt Switch, Port, lokalen Adapter, TTL und zuletzt gesehen; **Doppelklick oder Enter** öffnet die Details einschließlich sämtlicher TLVs. Über **Datei** PCAP importieren oder CSV/JSON/PCAP exportieren. Suche filtert die Tabelle, ohne Daten zu löschen.

## Adapter und Empfangsweg
Im Tab **Adapter** Empfangsweg Automatisch, Windows Packet Monitor oder Npcap einstellen. Windows Packet Monitor benötigt üblicherweise Adminrechte; die Kontextaktion **Als Admin starten** öffnet bewusst den UAC-Neustart. Npcap-Installation und native Diagnose liegen unter **Diagnose und Npcap**. Adapter in der Tabelle auswählen und **Auswahl übernehmen** drücken, wenn die Auswahl geändert wurde. Die Auswahl wird lokal gespeichert. **Aktualisieren** erkennt neu angeschlossene Adapter und stellt einen zuvor aktiven Empfang wieder her, soweit weiterhin Adapter ausgewählt sind.

## LLDP senden
Den Tab **Senden** öffnen, Adapter wählen und Systemname/Beschreibung/Portbeschreibung bearbeiten. **Sender vorbereiten** wählt Npcap und richtet nur den Empfang auf dem Adapter ein; es sendet **kein Paket**. Erst die separate Aktion **Senden einschalten** startet die Ankündigungen. Zum Ändern eines laufenden Profils zuerst stoppen; beim Stop wird LLDP-Shutdown versucht. Chassis-/Port-ID, TTL, Intervall, Ziel-MAC und zusätzliche TLVs stehen unter **Technische Einstellungen**. Paketvalidierung erfolgt vor dem Einschalten. Profile werden automatisch lokal gespeichert. Sender bleibt im Tray aktiv, aber niemals nach einem kompletten Neustart.

## Weitere Funktionen
Das Menü **Mehr** enthält **Protokoll**, **Einstellungen**, **Über WinLLDP** und **Beenden**. Protokoll als kompakte Tabelle; Doppelklick zeigt Details, Löschen braucht Bestätigung. Einstellungen: Sprache DE/EN/NL, System-/Hell-/Dunkelmodus, Tray-Verhalten und Speicherung des Protokolls. Über: Version, Lizenz und Drittanbieter.

Klick auf das Fenster-X minimiert bei aktiviertem Tray-Modus nur ins Tray, **Mehr → Beenden** beendet wirklich. Ohne funktionsfähiges Tray beendet das X das Programm. Alle Daten und Profile bleiben lokal.

## CLI
Die separaten `winlldp-cli-<version>-Windows-x64.exe`-Downloads können in der Konsole direkt gestartet werden. `--doctor` prüft Windows Packet Monitor, Npcap und Adapter; `--backend windows|npcap|auto` wählt den Empfang. PCAP-Offlineanalyse funktioniert ohne Capture-Treiber.

```powershell
.\winlldp-cli.exe --doctor
.\winlldp-cli.exe --backend windows --duration 60 --output neighbors.json
.\winlldp-cli.exe --read capture.pcap --format csv --output neighbors.csv
```
