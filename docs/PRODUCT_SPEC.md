# WinLLDP – Produktspezifikation

Stand: 04.11.2026. Interview abgeschlossen; autonome Fertigstellung beauftragt.

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

## Ausführungsentscheidungen
- Windows 10/11 x64; ARM und 32 Bit sind nicht Bestandteil dieses Releases.
- Rust mit nativer egui-Oberfläche, ohne Browser-/WebView- oder .NET-Laufzeitabhängigkeit.
- Deutsch, Englisch und Niederländisch; Standardsprache Deutsch.
- Automatischer Windows-Darkmode, zusätzlich hell/dunkel wählbar.
- Ein aktiver Sender läuft im Tray weiter. Nach vollständigem Prozessneustart ist Senden immer aus.
- CSV-, JSON- und klassischer Ethernet-PCAP-Export; PCAP-Import für Offlineanalyse.
- Npcap separat installieren; Standardzugriff ohne Erhöhung soweit die Treiberberechtigung dies erlaubt, UI-Aktion für administrativen Neustart.
- Ohne Capturetreiber kein erfundener Live-LLDP-Empfang; Adapterdiagnose und Offlineanalyse bleiben verfügbar.
- Empfang mehrerer ausgewählter Adapter gleichzeitig, Senden bewusst auf dem gewählten Adapter.
- Lokales Änderungsprotokoll bis 2000 Ereignisse; maximal 2048 Frames und 4096 Nachbarn im Arbeitsspeicher.
- MIT-Lizenz eigener Code, Drittanbieterhinweise separat; unsigned Release.
- Installation pro Benutzer ohne Windows-Dienst; kein automatischer Sendestart.

## Lieferziel
Versionierte Binärartefakte unter build/ einschließlich Versions-/Quellstand und Prüfsummen.
Portable EXE und Installationspaket; reproduzierbare Buildanleitung.
Implementierung, Tests und Buildnachweise werden in vier PRs geliefert.
