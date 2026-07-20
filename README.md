# raiOS Website & UI Lab

[raios.tech](https://raios.tech) ist die öffentliche Website von raiOS. Dieses Repository enthält außerdem das interaktive UI Lab, in dem die Oberfläche und ihre animierten Abläufe direkt im Browser erlebbar sind.

Der Branch `main` ist bewusst ein reines Website-Repository. Er enthält weder Kernel-Quellcode noch ein bootfähiges Betriebssystem oder eine QEMU-Umgebung.

## Inhalt

- **Website-Modus:** öffentliche Landingpage mit Projektgeschichte und Downloads/Links.
- **UI-Lab-Modus:** interaktive Vorschau der raiOS-Oberflächen.
- **Genesis-Film:** deterministisch gesteuerte Animation des Build- und Freigabeprozesses.
- **Surface-Demos:** Dream, Genesis, Setup, WLAN, Recovery, Vault und Personal Surface.
- **Cloudflare-Pages-Build:** schlanker Produktions-Build ohne Framework oder Paketmanager.

Zwischen Website und UI Lab kann direkt auf der Seite über den Modus-Schalter gewechselt werden.

## Lokal starten

Es müssen keine Abhängigkeiten installiert werden. Starte im Repository einen beliebigen statischen Webserver, zum Beispiel:

```powershell
python -m http.server 8080
```

Öffne anschließend:

```text
http://localhost:8080/raios-ui-lab.html
```

Das direkte Öffnen der HTML-Datei kann je nach Browser Einschränkungen bei Modulen oder Assets verursachen. Ein lokaler HTTP-Server ist daher vorzuziehen.

## Produktions-Build

```powershell
pwsh ./scripts/build-pages-site.ps1
```

Das Skript erzeugt `pages-dist/` und führt dabei grundlegende Prüfungen für referenzierte Dateien und die Cloudflare-Dateigrößenbegrenzung aus. Die Quelldatei `raios-ui-lab.html` wird im Build als `index.html` veröffentlicht.

## Deployment

Änderungen an den Website-Dateien auf `main` werden über [GitHub Actions](.github/workflows/deploy-raios-pages.yml) in das Cloudflare-Pages-Projekt `raios-site` deployt. Die produktive Domain ist [raios.tech](https://raios.tech).

Reine Dokumentationsänderungen lösen aufgrund der Pfadfilter im Workflow kein Deployment aus.

## Struktur

| Pfad | Zweck |
| --- | --- |
| `raios-ui-lab.html` | Gemeinsamer Einstiegspunkt für Website und UI Lab |
| `ui-lab/site/` | Website-Modus, Story, Boot-Sequenz und Genesis-Film |
| `ui-lab/lab/` | UI-Lab-Steuerung, Szenarien und Diagnose |
| `ui-lab/core/` | Gemeinsames Modell, Fonts und Zeichenprimitiven |
| `ui-lab/surfaces/` | Einzelne raiOS-Oberflächen und Abläufe |
| `ui-lab/assets/` | Audio-, Bild- und Oberflächen-Assets |
| `scripts/build-pages-site.ps1` | Reproduzierbarer Cloudflare-Pages-Build |
| `cloudflare/pages-worker.mjs` | Worker für die veröffentlichte Seite |

## Arbeiten am Projekt

- Markup und grundlegende Seitenstruktur liegen in `raios-ui-lab.html`.
- Website-Texte und Inszenierung liegen überwiegend in `ui-lab/site/`.
- Wiederverwendbare UI-Bausteine gehören nach `ui-lab/core/` oder `ui-lab/surfaces/`.
- Der Filmablauf wird zentral und deterministisch über `ui-lab/site/film.js` gesteuert.
- Neue Frameworks, Bundler oder Laufzeitabhängigkeiten sind für die aktuelle Architektur nicht vorgesehen.

Weitere verbindliche Hinweise für Coding-Agenten stehen in [AGENTS.md](AGENTS.md). Claude Code liest zusätzlich [CLAUDE.md](CLAUDE.md).

## Lizenz

Siehe [LICENSE](LICENSE).
