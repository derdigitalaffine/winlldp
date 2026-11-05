# Architektur und technische Grenzen

WinLLDP 1.0 verwendet Rust 2021 und Slint 1.17 für eine native Windows- und Linux-x64-Anwendung. Ein EXE enthält UI, Parser, Sender und Ressourcen. Keine eigene Hintergrunddienstinstallation.

## Module

- `protocol`: begrenztes Parsing/Encoding von Ethernet, bis zu zwei VLAN-Tags und LLDP-TLVs.
- `model`: adaptergetrennte Nachbarn, TTL/Shutdown, begrenzte Ereignisse und Framepuffer.
- `export`: CSV mit Schutz vor Formelinjektion, vollständiger JSON-Snapshot und klassischer Ethernet-PCAP.
- `capture`: dynamischer Npcap-Zugriff, je Adapter eigener Worker, explizite Sendersteuerung.
- `app`: mehrsprachige Oberfläche, Systemthema, Tray und Konfiguration.
- `install`: benutzerbezogene Installation/Deinstallation ohne Administratorpflicht.

## Standardzugriff und Administratorzugriff

Beide verwenden denselben installierten Npcap-Treiber. Npcap ohne Admin-only-Beschränkung kann normalen Benutzern Zugriff geben. Bei verweigertem Zugriff bietet die App einen bewussten UAC-Neustart an; der Sendestatus wird dabei nie wiederhergestellt. Ohne Npcap steht kein treiberfreier Live-Ethernetempfang zur Verfügung. Offline-PCAP-Auswertung bleibt möglich.

Npcap wird nicht mitgeliefert und nicht automatisch installiert. Es wird nur aus dem Windows-Systemverzeichnis geladen, nicht aus dem Arbeitsverzeichnis. Der Npcap-Kerneltreiber ist eine externe Voraussetzung; „ohne Dienst“ bezieht sich auf WinLLDP selbst.

Quellen: https://npcap.com/guide/npcap-users-guide.html und https://npcap.com/guide/npcap-devguide.html

## LLDP-Abdeckung

Alle TLV-Typen 0–127 werden verlustfrei mit Typ, Rohbytes und vorhandenen Interpretationen angezeigt. Pflicht-TLVs und Reihenfolge werden validiert. Standardfelder, Managementadressen, Fähigkeiten sowie verbreitete IEEE 802.1/802.3- und MED-TLVs erhalten zusätzliche benannte Felder. Unbekannte OUI/Subtypen bleiben vollständig einsehbar. Nicht jedes herstellerspezifische Binärformat kann semantisch interpretiert werden.

Der Sender bietet bearbeitbare Pflichtfelder, Textfelder und zusätzliche TLVs 7–127 als Hexdaten. Damit sind auch MED-, neue IEEE- und proprietäre Erweiterungen konfigurierbar. Ein Hexeditor ersetzt keine fachliche Prüfung eines proprietären Formats. MTU, Typ- und Längengrenzen werden geprüft. CDP ist kein Bestandteil von LLDP und wird nicht beworben.


Die Slint-Oberfläche ist in `ui/app-window.slint` definiert und verwendet die bestehenden Parser-, Monitor-, Settings- und Exporttypen. Linux startet Offline-PCAP und CLI; Windows aktiviert Packet Monitor/Npcap gemäß gewähltem Empfangsweg.
