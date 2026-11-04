# WinLLDP – Produktspezifikation

Stand: 04.11.2026. Anforderungsinterview läuft.

## Bestätigt
- Windows-Anwendung als hochwertiger LLDP-Client.
- Mindestens angeschlossenen Switch und Port anzeigen.
- Umfang: „alles was ein extrem guter LLDP-Client können soll“; konkrete Funktionen und Prioritäten werden im Interview präzisiert.
- Optional selbst LLDP senden/werben (vom Nutzer als „LLDP-Server“ bezeichnet).
- Sendebetrieb innerhalb der Anwendung ohne Windows-Dienst.
- Fertig kompilierte portable EXE und Installationspaket sollen im Repository als Build liegen.

## Noch offen
- Windows-Versionen und CPU-Architekturen.
- Treiber, Npcap-Abhängigkeit und Administratorrechte.
- Empfang/Senden je Adapter, automatische Adapterauswahl und mehrere Adapter gleichzeitig.
- Nachbardaten, LLDP-MED, herstellerspezifische TLVs und CDP.
- Inhalt und Konfiguration der eigenen LLDP-Ankündigungen.
- Live-Aktualisierung, Historie, Export und Diagnose.
- Oberfläche, Sprache, Hintergrund-/Tray-Verhalten.
- Installerumfang und optionaler Autostart.
- Signatur und konkrete Windows-Abnahme.

## Lieferziel
Versionierte Binärartefakte unter build/ einschließlich Versions-/Quellstand und Prüfsummen.
Portable EXE und Installationspaket; reproduzierbare Buildanleitung.
Aktuell existiert noch keine Implementierung und kein kompilierter Build.
