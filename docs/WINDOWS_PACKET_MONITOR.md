# Windows Packet Monitor: LLDP ohne Npcap

WinLLDP v1.1.0 bietet **echten Ethernet-LLDP-Empfang ohne Npcap-Installation** über den von Windows mitgelieferten Packet Monitor. Die Anwendung enthält eigenen LLDP-Parser, Nachbarzustandsverwaltung und UI. Windows stellt den vorhandenen Kernel-Erfassungsmechanismus bereit; WinLLDP installiert **keinen zusätzlichen Treiber**.

## Bedienung
1. **Adapter** öffnen. Unter **Empfang über** den Modus auswählen:
   - **Automatisch** (Standard): Erst Windows Packet Monitor versuchen; bei fehlender API, Zugriffsfehler oder nicht zuordenbarem Adapter Npcap als Fallback versuchen.
   - **Windows Packet Monitor**: Nur den integrierten Windows-Erfassungsweg verwenden; Npcap ist nicht nötig.
   - **Npcap**: Explizit Npcap verwenden, insbesondere wenn die Anwendung selbst LLDP senden soll.
2. Native Windows-Erfassung benötigt normalerweise **Administratorrechte**. In der Adapteransicht gibt es die Aktion **Als Admin neu starten** mit Windows-UAC. Automatisch umgeht WinLLDP die UAC-Abfrage nicht.
3. Passende aktive Schnittstellen auswählen, Empfang starten, Switch-Port-Einträge in der Tabelle per Doppelklick untersuchen.
4. Die Adaptertabelle zeigt das **tatsächlich aktivierte Backend**. Bei Fehlern die Windows-Diagnose oder `winlldp-cli.exe --doctor` verwenden.
5. Für **LLDP-Senden** auf Npcap umschalten und Npcap bei Bedarf aus der UI über den offiziellen Installer installieren. Der Sender ist standardmäßig aus.

CLI:
```powershell
winlldp-cli.exe --doctor
winlldp-cli.exe --backend windows --duration 60 --format json
winlldp-cli.exe --backend auto --duration 30 --output neighbors.json
winlldp-cli.exe --backend npcap --adapter "Ethernet" --send profile.json
```
Bei `--send` und `--backend auto` wählt die CLI automatisch Npcap. Eine explizite Kombination mit `--backend windows` wird abgelehnt.

## Technische Umsetzung
- Eingebundene Rust-Bibliothek `winptap` v0.2.0 (MIT/Apache-2.0), mit Packet-Monitor-Streaming-API und nativer Windows-x64-Ausweichimplementierung. Die zweite Variante verwendet einen nicht öffentlich dokumentierten, versionsabhängigen Windows-Treiberzugriff; sie ist deshalb nicht für jede Windows-Version garantiert.
- Capture erfolgt pro eindeutig identifiziertem Windows-Netzwerkadapter. Die Zuordnung bevorzugt GUID, danach eindeutige MAC oder Beschreibung; unklare Zuordnungen werden nicht geraten. Nur Ethernet-Frames mit LLDP EtherType 0x88CC werden verarbeitet; 802.1Q-Tags sind berücksichtigt.
- Eigene Frames (Quell-MAC) und als ausgehend erkannte Packet-Monitor-Frames werden verworfen, bevor die bestehenden LLDP-Nachbarn aktualisiert werden.
- Capture läuft in Arbeitsthreads mit begrenzter Queue; Beendigung und Neuinitialisierung geben Session-Ressourcen frei.
- Der Empfangsweg ist vom LLDP-Sender bewusst getrennt. Windows Packet Monitor erlaubt **keine Paketinjektion**; Npcap bleibt dafür nötig.

## Fehler 0x00000001 bei DeviceIoControl

Der Windows-Code `0x00000001` steht für `ERROR_INVALID_FUNCTION`, **nicht** für fehlende Administratorrechte (`ERROR_ACCESS_DENIED = 5`). Bei `winptap` kann der Fehler insbesondere im nativen `PktMonDev`-Ausweichbackend (`NativeIoctl`) auftreten, dessen nicht dokumentierte Steuerbefehle nicht auf jedem Windows-Build funktionieren. Ein erfolgreiches `--doctor` kann das reine Auflisten der Komponenten bestätigen, beweist aber nicht, dass der Live-Stream gestartet werden kann.

1. WinLLDP einmal **als Administrator** starten, den physischen Adapter auswählen und den Empfang nochmals prüfen. Wenn weiterhin `0x1` erscheint, liegt das Problem nicht allein an der UAC-Freigabe.
2. **Automatisch** auswählen, damit bei einem gescheiterten Windows-Stream auf ein installiertes Npcap zurückgefallen werden kann, oder explizit **Npcap** wählen. Packet Monitor und Npcap sind getrennte Empfangswege; der LLDP-Sender bleibt unabhängig davon aus, bis er ausdrücklich aktiviert wird.
3. Folgende Informationen für eine gerätespezifische Diagnose erfassen (PowerShell im Verzeichnis der CLI, ggf. installierter Name `winlldp-cli.exe`):

   ```powershell
   winver
   .\\winlldp-cli.exe --doctor --output pktmon-doctor.json
   .\\winlldp-cli.exe --backend windows --duration 10 --adapter "Ethernet"
   ```

   Den Adapternamen `Ethernet` durch den angezeigten Namen ersetzen. Der zweite Befehl prüft die echte Live-Erfassung und zeigt in der Konsole den Fehler und das verwendete Backend. Die Diagnose-JSON enthält Adapterkennungen und sollte vor dem Weitergeben auf sensible Netzwerkdaten geprüft werden.

Wenn die offizielle Windows-Streaming-API auf dem betreffenden System fehlt und `NativeIoctl` mit Fehler `1` abbricht, gibt es ohne weitere Daten keinen sicheren universellen IOCTL-Fix. Die erfolgreiche Nutzung von `pktmon.exe` garantiert **nicht**, dass die Bibliotheks-Streaming-API oder der NativeIoctl-Treiberpfad unterstützt wird. Für eine treiberunabhängige Eingrenzung kann man alternativ mit Microsofts `pktmon.exe` eine ETL-Datei aufzeichnen und in PCAPNG umwandeln; der aktuelle WinLLDP-PCAP-Import unterstützt PCAP und erfordert ggf. eine Konvertierung von PCAPNG nach PCAP.

## Einschränkungen und Akzeptanz
- Microsofts Packet-Monitor-API-Funktionen variieren je nach Windows-Build. Ein vorhandenes `pktmon.exe` bedeutet nicht, dass die Streaming-API verfügbar ist. Fehler werden sichtbar gemeldet und **nicht** als erfolgreiche Erfassung ausgegeben.
- Administratorrechte sind für den nativen Empfang erforderlich, sofern Windows die nötigen Privilegien nicht anderweitig erteilt.
- Auf Windows 10/11 x64 kann die interne Native-IOCTL-Ausweichimplementierung durch nicht dokumentierte Schnittstellen von Änderungen betroffen sein. Im Fehlerfall explizit Npcap wählen.
- Die automatischen Windows-CI-Tests belegen Build, Unit-Tests, Parser und Smoke-Tests, **nicht** echten LLDP-Datenverkehr. Hardwaretest: physischen LLDP-Switch an ausgewählten Adapter anschließen; Switchname, Chassis, Port, TTL, VLAN-Tags, eigene Echo-Filterung und Ablauf bei Kabeltrennung kontrollieren.
- WinLLDP benötigt weiterhin entsprechende Netzwerkberechtigungen. WinLLDP selbst stellt keinen Windows-Dienst oder zusätzlichen Treiber bereit.

Quellen:
- Microsoft Packet Monitor: https://learn.microsoft.com/en-us/windows-server/networking/technologies/pktmon/pktmon-syntax
- Microsoft Packet Monitor API: https://learn.microsoft.com/en-us/windows/win32/pktmon/pktmon
- winptap: https://github.com/shellrow/winptap
