# WinLLDP – Produktspezifikation

Stand: 04.11.2026. Anforderungsinterview läuft.

## Bestätigt
- Windows-Anwendung als hochwertiger LLDP-Client für Windows 10 und Windows 11.
- Komfortabel in der UI umschaltbarer Betrieb ohne Administratorrechte sowie mit Administratorrechten/Npcap. Der tatsächlich mögliche Funktionsumfang je Modus muss technisch geprüft und sichtbar dargestellt werden.
- Die UI hat höchste Priorität: hochwertige, komfortable Bedienung und Zugriff auf alle empfangenen LLDP-Daten. Auch nicht interpretierte Erweiterungen sollen zur Inspektion zugänglich bleiben.
- Mindestens angeschlossenen Switch und Port anzeigen.
- Umfang: „alles was ein extrem guter LLDP-Client können soll“; konkrete Funktionen und Prioritäten werden im Interview präzisiert.
- Optional selbst LLDP senden/werben (vom Nutzer als „LLDP-Server“ bezeichnet).
- Sendebetrieb innerhalb der Anwendung ohne Windows-Dienst.
- Fertig kompilierte portable EXE und Installationspaket sollen im Repository als Build liegen.

## Noch offen
- CPU-Architekturen.
- Technischer Umfang und Voraussetzungen des Betriebs ohne Administratorrechte sowie des Betriebs mit Administratorrechten/Npcap.
- Empfang/Senden je Adapter, automatische Adapterauswahl und mehrere Adapter gleichzeitig.
- Detaillierte Darstellung aller LLDP-Daten einschließlich LLDP-MED und herstellerspezifischer Erweiterungen; zusätzliches CDP ist noch unbestätigt.
- Inhalt und Konfiguration der eigenen LLDP-Ankündigungen.
- Live-Aktualisierung, Historie, Export und Diagnose.
- Oberfläche, Sprache, Hintergrund-/Tray-Verhalten.
- Installerumfang und optionaler Autostart.
- Signatur und konkrete Windows-Abnahme.

## Lieferziel
Versionierte Binärartefakte unter build/ einschließlich Versions-/Quellstand und Prüfsummen.
Portable EXE und Installationspaket; reproduzierbare Buildanleitung.
Aktuell existiert noch keine Implementierung und kein kompilierter Build.
