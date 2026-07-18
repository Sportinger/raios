# REPO CLEANUP — Auftrag für den Aufräum-Agenten (Repo-Root)

> **Prinzip:** Beweis statt Bauchgefühl. Nichts wird gelöscht, weil es "tot aussieht" —
> nur, weil drei Beweise es belegen. Alles Zweifelhafte geht in Quarantäne
> (`_attic/`), nicht in den Müll. **Nach jedem Verschiebe-Schritt muss der
> Build grün bleiben** — sonst sofort zurück.
>
> Modus: **konservativ.** Eine Lane, ein Branch (`repo-cleanup`), kleine Commits.

---

## Phase 1 — Beweise sammeln (nichts anfassen)

Für JEDEN Root-Ordner und jedes lose File eine Zeile in `cleanup-evidence.md`:

| Pfad | Workspace-Member? | Referenziert von? | Letzter Commit | Urteil |
|---|---|---|---|---|

Beweisquellen:
1. `cargo metadata --format-version 1` → welche Crates sind Workspace-Member
   bzw. Dependencies (auch via `path = ...` in Cargo.toml-Dateien)
2. Referenz-Suche: `rg -l "<ordnername>" --glob '!target'` über Repo, Scripts,
   `.github/workflows`, Cargo-Files — wer erwähnt den Pfad?
3. `git log -1 --format="%ci %h %s" -- <pfad>` → letzte echte Änderung

Urteils-Kategorien:
- **AKTIV** — Workspace-Member ODER von Build/CI/Scripts referenziert
- **INFRA** — `.github`, `.gitignore`, `.gitattributes`, `scripts`, `LICENSE`,
  `Cargo.toml`/`.lock`, `CLAUDE.md`, `AGENTS.md`, `README.md`, `docs/`
- **GENERIERT** — Build-Output, reproduzierbar (`target/`, `.cargo-home`,
  ggf. `release/`, `vendor/` falls `cargo vendor`-Output)
- **VERDÄCHTIG** — keine Referenzen + letzter Commit vor 01.06.2026
- **UNKLAR** — Beweise widersprechen sich → NICHT anfassen, im Report an Owner

## Phase 2 — Owner-Review (Pflicht-Stopp)

`cleanup-evidence.md` committen, Owner liest die Urteile, streicht/bestätigt.
**Ohne dieses Review keine Phase 3.** Vorab-Verdachtsliste aus dem
Datums-Muster (10.05. = alte Projektphase), vom Agenten zu VERIFIZIEREN,
nicht zu übernehmen: `fake-cloud`, `modules`, `ota`, `registry`,
`seed-runtime`, `ci` (vs. `.github`?), `.vscode`, `device-protocol`,
`usb-*.txt` (6 lose Files, vermutlich Installations-Notizen von Mai).

## Phase 3 — Umsetzen (ein Commit pro Zeile)

- **GENERIERT** → prüfen, dass es in `.gitignore` steht; falls versioniert:
  aus Git entfernen (das ist die EINZIGE erlaubte "Löschung" — reproduzierbar).
- **VERDÄCHTIG (bestätigt)** → `git mv` nach `_attic/<name>`. `_attic/` steht
  in einer README als "Quarantäne — nichts hier wird gebaut; nach 30 Tagen
  ohne Vermissen darf der Owner löschen".
- **usb-*.txt** → Inhalt sichten: noch gültige Anleitung → in
  `docs/architecture/hardware/` einarbeiten; Rest → `docs/_archive/` (datiert).
- **AKTIV / INFRA / UNKLAR** → bleibt exakt, wo es ist.
- Root-Ziel danach: nur AKTIV-Crates, INFRA, `_attic/`.

## Phase 4 — Verifizieren (das Gate)

Nach JEDEM Verschiebe-Commit, und einmal am Ende komplett:
- [ ] `cargo build` (Workspace) grün
- [ ] `cargo test -p raios-core` grün
- [ ] QEMU-Smoke-Profil läuft durch, `raios.vm_test_report.v0` entsteht
- [ ] `.github`-CI-Workflow lokal nachvollzogen: referenziert er verschobene Pfade?
- [ ] `rg "_attic"` außerhalb von `_attic/` selbst → 0 Treffer
Ein roter Check → letzten Commit revertieren, Pfad als UNKLAR markieren, weiter.

## Abschluss

- `cleanup-evidence.md` → `docs/_archive/2026-07-18_repo-cleanup-evidence.md`
- HANDOFF-Block aktualisieren; Merge-Freigabe durch Orchestrator/Owner
- Offene UNKLAR-Fälle als Liste an den Owner (max. 10, priorisiert)
