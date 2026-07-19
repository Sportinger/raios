# raiOS Dream UI-Lab

Interaktive Browser-Referenz der raiOS-Genesis-Shell. In diesem Repository wird
die UI entworfen und überprüft; die spätere Übernahme in den echten Kernel ist
ein eigener, abgesicherter Umsetzungsschritt.

## Öffnen

```powershell
Start-Process .\raios-ui-lab.html
```

Die kanonische Zeichenfläche ist **1920×1080 physisch** beziehungsweise
**960×540 logisch** bei Skalierung 2. Auf kleineren Browserfenstern wird sie nur
für die Anzeige herunterskaliert. `F11` zeigt sie auf einem Full-HD-Display
pixelgenau. Nach Änderungen genügt `F5`.

**Arbeitsrepo:** `raios2-dream`

**Renderer-Referenz:** raiOS `ad07424` (`dream.rs`, `genesis.rs`,
`genesis_layout.rs`, `framebuffer.rs`)

## Wahrheitsstatus

- Die responsive Genesis- und Dream-Geometrie ist aus dem Rust-Renderer
  übertragen. Eingebaute Selbsttests pinnen sowohl 1280×800 als auch
  1920×1080.
- Dream-Keyframes, Klammergrenzen, Composer-Anker, Hintergrunddither,
  auflösungsabhängige Sternenzahl und Kerzen-Bodenanker folgen dem aktuellen
  Renderer.
- Chatantwort, WLAN-Netze, Recovery-Werte und Build-Fortschritt sind lokale
  Fixtures. Sie demonstrieren echte UI-Zustände, sind aber keine Live-Daten.
- Neue Performance-Ideen erscheinen ausschließlich als **PROPOSAL** und dürfen
  nicht als bereits im Kernel umgesetzt beschrieben werden.

## ACTUAL und PROPOSAL

Standardmäßig zeigt das Lab die saubere UI ohne Diagnose-Chrome.

```text
?diagnostics=1        Aktuellen Renderpfad und Framekosten anzeigen
?mode=proposal        Geplante Damage-Domänen klar markiert simulieren
```

Tastenkürzel im fokussierten Canvas:

- `F2`: Katalog aller gespiegelten UI-Zustände öffnen
- `Ctrl+Shift+M`: zwischen ACTUAL und PROPOSAL wechseln
- `Ctrl+Shift+D`: Diagnose ein-/ausblenden
- `Ctrl+Shift+E`: Design-Delta als JSON kopieren

Der Toggle **Website** oben rechts wechselt in eine kurze Präsentationsseite.
Sie verwendet dieselbe laufende Canvas-Vorschau. Der Surface-Compositor setzt
das originale 4096×2304-Hardwarefoto, den gemessenen Displayausschnitt und das
passgenaue Reflexions-Overlay zusammen. Das 16:9-UI bleibt im fotografierten
16:10-Panel unverzerrt und erhält deshalb schmale schwarze Letterbox-Balken.

Der aktuelle Pfad bleibt ehrlich ausgewiesen: gebackener Hintergrund in den
Backbuffer, anschließend vollständiger Present; Mauszeiger und Textcursor
liegen auf der kleinen Front-Ebene. `DamageSet<32>`, drei feste
Kompositionsebenen und der aktive 60-Hz-Deadline-Scheduler sind noch Vorschläge.

## Struktur und vollständiger Zustandskatalog

`raios-ui-lab.html` ist nur noch der dünne Einstieg. Die Renderer liegen unter
`ui-lab/` in denselben Kategorien wie die aktuelle Rust-UI: Genesis, Dream,
Recovery, WiFi, Secret Vault und Personal-Surface. Der mit `F2` erreichbare
Katalog enthält die visuellen Zustandsklassen dieser Host-Flows, sämtliche
Vault-Ergebnisse sowie die aktuellen Personal-Surface-Programme Calculator und
Editor. Die genaue Zuordnung steht in `ui-lab/README.md`.

Direktlinks für reproduzierbare Ansichten verwenden `?scenario=<id>`, zum
Beispiel `?scenario=vault.managing` oder `?scenario=personal.editor`.
Der Website-Modus kann direkt mit `?site=1` geöffnet werden.

## Renderer-Gesetze

- Eine 8×8-Bitmap-Schrift, Vorschub 9; chunky bei Skalierung 2 und hi-res mit
  einem physischen Pixel pro Fontpixel.
- Nur die Kernel-Palette; keine runden Ecken, Verläufe oder Antialiasing.
- Keine erfundene Aktion darf echte Autorität vortäuschen.
- Sichere Overlays, Focus und finaler Present bleiben core-eigen.
- Neue Panels und Buttons bleiben unverdrahtet, bis der echte Kernelpfad
  separat umgesetzt und negativ getestet wurde.

## Prüfen und Screenshots

```powershell
.\verify-ui-lab.ps1
.\verify-ui-lab.ps1 -Query "suite=all"
.\shot.ps1 -Out shot.png
.\shot.ps1 -Query "scenario=genesis.dream.chat&chat=demo" -Out open.png
.\shot.ps1 -Query "mode=proposal&scenario=genesis.dream.chat" -Out proposal.png
.\shot.ps1 -Query "scenario=genesis.setup" -Out setup.png
.\shot.ps1 -Query "scenario=wifi.password.vault" -Out wifi.png
.\shot.ps1 -Query "scenario=vault.managing" -Out vault.png
.\shot.ps1 -Query "scenario=personal.editor" -Out editor.png
.\shot.ps1 -Query "site=1&scenario=genesis.dream.chat" -Out website.png
```

`shot.ps1` erzeugt standardmäßig echte 1920×1080-PNGs. Nach jeder Änderung
muss mindestens der Selbsttest grün sein und der relevante Screenshot visuell
kontrolliert werden. Lab-Schalter werden in automatischen Screenshots
standardmäßig ausgeblendet; `-IncludeLabChrome` nimmt sie sichtbar mit auf.

## Treue-Klassen und Rückkanal

- **code-treu:** Geometrie oder Darstellung ist aus dem aktuellen Rust-Code
  gespiegelt und durch einen Predicate-Wert abgesichert.
- **simuliert:** Zustand existiert real, wird im Lab aber mit einem Fixture
  gespeist.
- **PROPOSAL:** noch nicht gelandete Architektur oder Interaktion; immer sichtbar
  markiert und im Design-Delta mit `wired: false` geführt.

`Ctrl+Shift+E` exportiert Baseline, Tokens, responsive Layouts und den
`wired`-Status. Langfristig soll derselbe no-dep-Rust-Renderer sowohl Kernel als
auch Browser-Wasm bedienen; dann ersetzt Pixelgleichheit die manuelle Portierung.

## GitHub

JavaScript läuft nicht direkt innerhalb einer gerenderten GitHub-README. Die
README kann jedoch einen großen Screenshot und einen Link zur interaktiven Demo
enthalten. [GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages)
kann diese HTML/CSS/JavaScript-Datei direkt als Website ausliefern. Das
Repository bleibt bis zu einer ausdrücklichen
Owner-Entscheidung lokal; es wird nicht automatisch veröffentlicht.

## QEMU-Vergleich

Für eine interaktive QEMU-Vorschau immer `scripts/run-stage0-baremetal-vm.ps1`
mit einer temporären Kopie von `release/raios-stage0.img` verwenden. Der Wrapper
aktiviert `qemu-xhci`, `usb-kbd` und `usb-tablet`. Vorher prüfen, dass keine
zweite QEMU-Instanz läuft.
