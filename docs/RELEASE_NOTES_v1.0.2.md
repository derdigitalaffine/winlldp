# WinLLDP v1.0.2 — LLDP-Empfang, Npcap-Diagnose und kompakte Oberfläche

## Empfang und Diagnose
- Mit eigener Quell-MAC markierte LLDP-Sende-Echos werden nicht mehr als fremde Switch-Nachbarn dargestellt.
- Nach Möglichkeit wird die Capture-Richtung auf **eingehend** gesetzt; unterstützt die Npcap-Version diese Einstellung nicht, bleibt der lokale MAC-Filter aktiv.
- Npcap-DLL und -Version, sichtbare Capture-Geräte und der konfigurierbare Admin-only-Status werden geprüft; Adapterfehler erscheinen in der Diagnose.
- CLI-Option `--doctor` liefert Diagnoseinformationen als JSON.
- Bei fehlendem Treiber gibt es einen sichtbaren Installationshinweis und einen direkten Weg über die UI zur offiziellen Downloadseite und zum Windows-UAC-Start des manuell heruntergeladenen Installers.
- Wenn Npcap nur Administratorzugriff erlaubt, kann WinLLDP aus der Adapteransicht bewusst erhöht neu gestartet werden.

## Kompaktes UI
- Kleineres Startfenster, schlankere Titelleiste und Navigation.
- Keine großen Dashboard-Kacheln oder permanent offenen TLV-Detailbereiche auf der Nachbarseite.
- Nachbarn in einer tabellarischen Fünfspaltenliste; **Doppelklick oder Enter** öffnet die vollständigen LLDP-Daten in einem eigenen Fenster.
- Adapter ebenfalls in einer kompakten Status-Tabelle statt gestapelter Karten.
- Drei Sprachen, Hell-/Dunkelmodus, Export und der standardmäßig deaktivierte Sender bleiben erhalten.

## Lieferumfang
Windows 10/11 x64: portable GUI-EXE, CLI-EXE, Inno-Setup-Installer, SHA-256-Prüfsummen und Buildinformationen.

**Wichtig:** Npcap wird aus Lizenzgründen **nicht** mitgeliefert und wird nicht ohne Einwilligung installiert. Der Nutzer lädt den signierten Installer von npcap.com und startet ihn aus der UI. Keine Codesignatur für WinLLDP-Binärdateien. Auf echter Windows-Hardware mit physischem LLDP-Switch muss die Npcap-/Capture-Funktion weiterhin verifiziert werden. CI kann nur Windows-Kompilierung, automatische Tests, Diagnose-CLI, Selbsttest und Installer-Paketierung belegen.

Weitere Details: [Npcap-Installation und Diagnose](NPCAP_DIAGNOSTICS.md).
