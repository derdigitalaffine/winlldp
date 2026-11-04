# WinLLDP – Produktspezifikation

Stand: 04.11.2026. Anforderungsinterview läuft.

## Bestätigt
- Windows-Anwendung als hochwertiger LLDP-Client für Windows 10 und Windows 11.
- Komfortabel in der UI umschaltbarer Betrieb ohne Administratorrechte sowie mit Administratorrechten/Npcap. Der tatsächlich mögliche Funktionsumfang je Modus muss technisch geprüft und sichtbar dargestellt werden.
- Die UI hat höchste Priorität: hochwertige, komfortable Bedienung und Zugriff auf alle empfangenen LLDP-Daten. Auch nicht interpretierte Erweiterungen sollen zur Inspektion zugänglich bleiben.
- Mindestens angeschlossenen Switch und Port anzeigen.
- Umfang: „alles was ein extrem guter LLDP-Client können soll“; konkrete Funktionen und Prioritäten werden im Interview präzisiert.
- Optional selbst LLDP senden/werben (vom Nutzer als „LLDP-Server“ bezeichnet).
- Sendebetrieb standardmäßig ausgeschaltet; bewusst in einem eigenen Programmbereich einschaltbar.
- Windows-Gerätename und Adapterdaten für die eigenen Ankündigungen automatisch vorbelegen, vor dem Senden bearbeitbar machen.
- Auch beim Senden soll der vollständige LLDP-Funktionsumfang konfigurierbar sein. Die konkrete Liste unterstützter Standard-, LLDP-MED- und herstellerspezifischer TLVs wird technisch ausgearbeitet; „vollständig“ ist durch eine überprüfbare Funktionsmatrix zu konkretisieren.
- Sendebetrieb innerhalb der Anwendung ohne Windows-Dienst.
- Fertig kompilierte portable EXE und Installationspaket sollen im Repository als Build liegen.

## Oberfläche und Bedienung
- Hauptansicht: Switch und Port prominent, Adapter-/Nachbarliste sowie Detailbereich für sämtliche LLDP-Felder.
- Verhalten beim Schließen in den Einstellungen wählbar; standardmäßig in den Infobereich (Tray) minimieren.
- Änderungsprotokoll für Portwechsel, neue Nachbarn und Verbindungsverlust sowie Datenexport.
- Eigener Programmbereich für den standardmäßig ausgeschalteten LLDP-Sendebetrieb.
- Über-Seite mit „Entwickelt von Dominik Tröster, Verbandsgemeinde Otterbach-Otterberg“ und Hinweis auf die MIT-Lizenz.
- Eigenes Appsymbol: Nachrichtendienst-Otter, der an einem RJ45-Port lauscht. Das Motiv auch klein in die Oberfläche integrieren.
- MIT-Lizenz für den eigenen Projektcode; Hinweise und Lizenzen eingebundener Fremdkomponenten separat erhalten.

## Noch offen
- CPU-Architekturen.
- Technischer Umfang und Voraussetzungen des Betriebs ohne Administratorrechte sowie des Betriebs mit Administratorrechten/Npcap.
- Empfang/Senden je Adapter, automatische Adapterauswahl und mehrere Adapter gleichzeitig.
- Detaillierte Darstellung aller LLDP-Daten einschließlich LLDP-MED und herstellerspezifischer Erweiterungen; zusätzliches CDP ist noch unbestätigt.
- Konkrete TLV-Funktionsmatrix für eigene LLDP-Ankündigungen; Profile und weitere Komfortfunktionen noch offen.
- Details zu Live-Aktualisierung, Aufbewahrung des Änderungsprotokolls, Exportformaten und Diagnose.
- UI-Details, Sprache und Verhalten eines aktiven Sendebetriebs im Tray bzw. nach einem Neustart.
- Installerumfang und optionaler Autostart.
- Signatur und konkrete Windows-Abnahme.

## Lieferziel
Versionierte Binärartefakte unter build/ einschließlich Versions-/Quellstand und Prüfsummen.
Portable EXE und Installationspaket; reproduzierbare Buildanleitung.
Aktuell existiert noch keine Implementierung und kein kompilierter Build.
