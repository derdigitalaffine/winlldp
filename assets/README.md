# WinLLDP-Appsymbol

Motiv: ein Nachrichtendienst-Otter, der an einer RJ45-Buchse lauscht.

## Quelldatei und Erzeugung

`winlldp-otter.svg` enthält die generierte Rastergrafik als eingebettetes transparentes PNG mit 512 × 512 Pixeln. Sie ist keine reine Vektorgrafik.
Erstellt mit dem integrierten Imagegen-Werkzeug am 04.11.2026.

## Windows- und UI-Dateien erzeugen

```sh
python -m pip install Pillow
python scripts/generate_icons.py
```

Erzeugt `assets/winlldp.ico` mit 16, 20, 24, 32, 40, 48, 64, 128 und 256 Pixeln,
das PNG in Quellgröße und kleine PNGs mit 24, 32, 48 und 64 Pixeln.
Der Anwendungsbuild muss diesen Schritt vor dem Einbetten des Icons ausführen.

## Vorgesehene Einbindung

- EXE und Installer: `assets/winlldp.ico`.
- Kopfbereich der Anwendung: kleines Otter-Motiv neben WinLLDP.
- Tray: kleine Iconvariante; Empfangs-/Sendestatus zusätzlich durch Tooltip und Text in der Anwendung vermitteln.
- Über-Seite: größeres Motiv, Urheberangabe und MIT-Lizenzhinweis.
- Bei 100 % und 200 % Windows-Skalierung sowie auf hellem und dunklem Hintergrund prüfen.

Aktueller Stand: Grafik und Konvertierung vorhanden; eine laufende Anwendungsoberfläche existiert noch nicht.

## Generierungsprompt

Use case: logo-brand. Asset type: premium Windows desktop app icon and small in-app mascot for WinLLDP. Create a single polished, memorable icon: an anthropomorphic intelligence-agent otter listening intently at an RJ45 Ethernet socket. The charming clever otter has recognizable small round ears, broad pale muzzle, whiskers, dark brown fur and a discreet secret-agent earpiece; one paw is cupped to its ear as it leans toward a clearly recognizable metallic rectangular RJ45 female network port with eight gold contacts and a small green link light. Head and upper body with port form one compact strong silhouette. Professional sophisticated playful technology mascot, beautifully crafted crisp stylized illustration with subtle dimensional shading, clean bold contours, minimal tiny detail, high contrast, legible at 32px as well as beautiful enlarged. Square composition, centered with generous safe padding; true transparent background outside subject. No text, no letters, no watermark, no surrounding scene, no weapons, no official seals.

## Lizenz

Die eigenen Projektdateien unterliegen der MIT-Lizenz im Repository. Fremdkomponenten behalten ihre jeweiligen Lizenzbedingungen.
