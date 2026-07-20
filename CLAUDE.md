# Claude Code Instructions

Lies zuerst vollständig [AGENTS.md](AGENTS.md). Diese Datei ist die verbindliche und ausführliche Arbeitsanweisung für alle Coding-Agenten.

## Kurzfassung

- `main` enthält nur die raiOS-Website und das eingebettete UI Lab, keine Betriebssystem-Implementierung.
- Bewahre die statische HTML/CSS/JavaScript-Architektur ohne zusätzliche Frameworks oder Abhängigkeiten.
- Verwende bestehende Bausteine aus `ui-lab/core/`, `ui-lab/site/` und `ui-lab/surfaces/` erneut.
- Halte den Genesis-Film über `ui-lab/site/film.js` deterministisch und beim Springen in der Timeline korrekt.
- Prüfe geändertes JavaScript mit `node --check` und führe vor der Übergabe den Build aus:

```powershell
pwsh ./scripts/build-pages-site.ps1
```

- Verändere keine fremden Arbeitsstände, schreibe keine Secrets fest und committe oder pushe nur auf ausdrücklichen Wunsch.
