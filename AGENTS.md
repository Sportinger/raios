# Arbeitsanweisungen für Coding-Agenten

## Projektgrenze

Der Branch `website` ist ausschließlich die öffentliche raiOS-Website mit eingebettetem UI Lab. Kernel, Bootloader, Rust-Workspace, QEMU-Images, Toolchains und sonstige Betriebssystem-Implementierungen gehören nicht in diesen Branch.

Keine entfernten OS-Bestandteile neu anlegen oder aus anderen Branches übernehmen, sofern der Nutzer dies nicht ausdrücklich verlangt.

## Vor jeder Änderung

1. `README.md` lesen und die bestehende Architektur respektieren.
2. Mit `git status --short --branch` prüfen, ob bereits Änderungen vorhanden sind.
3. Fremde oder nicht zum Auftrag gehörende Änderungen unverändert lassen.
4. Die betroffenen Dateien und vorhandenen Komponenten zuerst untersuchen, bevor neue Strukturen angelegt werden.

## Architektur

- `raios-ui-lab.html` ist der gemeinsame Einstiegspunkt für Website und UI Lab.
- `ui-lab/site/` enthält Website-Modus, Story, Boot-Sequenz und Filmsteuerung.
- `ui-lab/lab/` enthält UI-Lab-Steuerung, Szenarien und Diagnostik.
- `ui-lab/core/` enthält gemeinsam verwendete Modelle und Zeichenprimitiven.
- `ui-lab/surfaces/` enthält die einzelnen Oberflächen und Abläufe.
- `ui-lab/assets/` enthält ausschließlich tatsächlich verwendete Medien.
- `scripts/build-pages-site.ps1` erzeugt den Inhalt von `pages-dist/`.

## Regeln für Änderungen

- Änderungen klein und auf den Auftrag begrenzt halten.
- Vorhandene Primitiven, Komponenten und Animationstechniken wiederverwenden.
- Website-Modus, UI-Lab-Modus und den Modus-Schalter funktionsfähig halten.
- Zeitabhängige Filmabläufe zentral und deterministisch über `ui-lab/site/film.js` steuern. Keine davon unabhängigen Animationen einführen, die beim Suchen oder Zurückspringen in der Timeline einen falschen Zustand zeigen.
- Die bestehende statische Architektur ohne Framework, Bundler oder Paketmanager beibehalten, solange der Nutzer keinen Architekturwechsel verlangt.
- Keine generierten Build-Dateien aus `pages-dist/` committen.
- Keine ungenutzten Assets hinzufügen. Einzelne Dateien dürfen für Cloudflare Pages nicht größer als 25 MiB sein.
- Responsive Darstellung, ausreichenden Kontrast, Tastatur-Fokus und reduzierte Bewegung berücksichtigen.
- Zugangsdaten, Tokens, Account-IDs und andere Secrets niemals in Dateien oder Ausgaben festschreiben.

## Prüfung

Nach Änderungen mindestens die für den Umfang passenden Prüfungen ausführen:

```powershell
node --check <geänderte-js-datei>
pwsh ./scripts/build-pages-site.ps1
```

Bei visuellen Änderungen die Seite zusätzlich über einen lokalen HTTP-Server im Browser prüfen. Dabei Website- und UI-Lab-Modus sowie relevante Timeline-Positionen testen.

Für dieses Website-Repository ist keine QEMU-Prüfung erforderlich. QEMU- oder Bare-Metal-Skripte gehören nicht zum Branch `website`.

## Git und Deployment

- Nur die konkret bearbeiteten Dateien stagen.
- Keine fremden Änderungen verwerfen, zurücksetzen oder überschreiben.
- Nicht committen oder pushen, außer der Nutzer verlangt es ausdrücklich.
- Ein Push relevanter Website-Pfade auf `website` startet den Workflow `.github/workflows/deploy-raios-pages.yml` und kann die produktive Website verändern. Der Workflow deployt dabei ausdrücklich den Cloudflare-Pages-Branch `website`.
- Vor einem angeforderten Push muss der Produktions-Build erfolgreich durchlaufen.

## Dokumentation

Wenn sich Einstieg, Architektur, Build oder Deployment ändern, `README.md` und `AGENTS.md` konsistent aktualisieren.
