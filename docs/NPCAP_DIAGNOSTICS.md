# Npcap unter Windows: Installation und Fehlersuche

WinLLDP verwendet einen eigenen LLDP-Parser/Encoder und benötigt **keinen WinLLDP-Windows-Dienst**. Für das Lesen/Senden von Layer-2-Ethernet-Frames auf Windows 10/11 braucht die Anwendung allerdings eine zulässige Capture-Schnittstelle. Npcap stellt einen dafür geeigneten signierten NDIS-Treiber und `wpcap.dll` bereit. WinLLDP kann diesen Kernel-Treiber nicht durch normale Windows-Sockets ersetzen.

## Installation direkt aus WinLLDP starten
1. In WinLLDP **Adapter** öffnen. Der Npcap-Bereich zeigt Version, sichtbare Geräte und ggf. eine Administratorbeschränkung sowie bekannte Fehler.
2. **1. Npcap offiziell herunterladen** öffnet https://npcap.com/#download im Browser. Die Datei dort direkt herunterladen.
3. **2. Installer auswählen und starten …** öffnet einen Dateidialog. Die zuvor von npcap.com geladene, signierte `npcap-*.exe` auswählen. WinLLDP startet sie mit Windows **UAC / Als Administrator ausführen**. Installation und Lizenzbestätigung bleiben im Hersteller-Assistenten; kein Silent-Setup.
4. Erst nach erfolgreichem Abschluss **3. Erneut prüfen** anklicken. Danach **Empfang starten** bzw. Adapterauswahl anwenden. Falls erforderlich Windows/WinLLDP neu starten.
5. Falls Npcap mit **Admin-only** installiert wurde oder der Zugriff verweigert bleibt: Im Adapterbereich **Mit Administratorrechten neu starten …** auswählen und UAC bestätigen. Der optionale Sender bleibt nach einem Neustart aus.

WinLLDP verteilt den kostenlosen Npcap-Installer **nicht** mit, lädt ihn nicht unsichtbar herunter und behauptet keinen erfolgreichen Treiberinstallationsabschluss, bloß weil der Assistent gestartet wurde. Lizenzhinweise und Voraussetzungen: https://npcap.com/guide/npcap-users-guide.html sowie https://npcap.com/guide/ .

## Diagnose
- In der UI zeigt **Adapter** den Ergebnistext von `pcap_lib_version`, die Liste aus `pcap_findalldevs` und mögliche Fehler je Adapter. Die Bibliothek wird nur aus dem Windows-Systemverzeichnis geladen.
- CLI: `winlldp-cli.exe --doctor` gibt einen JSON-Bericht zur Npcap-API und Windows-Adapterauflistung aus, auch wenn Npcap fehlt. Alternativ: `winlldp-cli.exe --list-adapters`.
- **Npcap fehlt**: Treiber installieren wie oben; Adapter allein sind nicht mit echtem Capture gleichzusetzen.
- **Npcap vorhanden, aber kein Capture**: Prüfen, ob der Treiber läuft, der ausgewählte Ethernetadapter Link hat, die Access-Control-Liste von Npcap Admin-only erzwingt und ob ein erneuter Start als Administrator hilft.
- **Capture läuft, aber nur eigene Frames / keine Switch-Nachbarn**: WinLLDP blendet seit v1.0.2 LLDP-Frames mit der eigenen Quell-MAC aus und bevorzugt ankommende Pakete. Switch-Port auf aktiviertes LLDP, physische Verbindung sowie Port-Empfang prüfen. Ohne fremde LLDP-Ankündigungen sind **0 Nachbarn** korrekt, auch wenn der Treiber funktioniert.
- **Nur Offline-PCAP vorhanden**: Die Daten stammen aus der Datei und beweisen keine funktionierende Live-Verbindung.
- **Physischer Nachweis**: Windows-PC mit Npcap an einem LLDP-Switch-Port anschließen, per Paketmitschnitt die LLDP-Multicast-Pakete 01:80:C2:00:00:0E/EtherType 0x88CC prüfen, Quell-MAC und angezeigte Switch-/Portdaten vergleichen.

## Sicherheit, Grenzen und Lizenz
Der Anwender wählt die Setup-Datei selbst. Sie muss von npcap.com stammen; vor der UAC-Freigabe ist der Herausgeber/die Authenticode-Signatur im Windows-Dialog zu kontrollieren. Die einfache Dateinamensprüfung allein ist **keine Signaturprüfung**. Eine automatische oder unbeaufsichtigte Installation erfolgt nicht.

Die freie Npcap-Version ist laut Herstellerlizenz nur in den ausdrücklich erlaubten Nutzungsfällen zu verwenden; Umverteilung und OEM-Silentinstallation bedürfen zusätzlicher Rechte. WinLLDP verschiebt diese Bedingungen nicht durch seine MIT-Lizenz. Für höhere Zuverlässigkeit ersetzt ein funktionierender Npcap-Treiber nicht die Prüfung, ob der Switch tatsächlich LLDP sendet.
